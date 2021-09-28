import {
  EditorState,
  NodeSelection,
  Plugin,
  Transaction,
} from "prosemirror-state";
import { Node as ProsemirrorNode, Schema } from "prosemirror-model";
import { Command } from "prosemirror-commands";
import { keymap } from "prosemirror-keymap";

type WrapperNodes = [number, ProsemirrorNode<Schema>[]];
type WrapperNodesList = WrapperNodes[];

const getRangeForNodeAtMappedPosition = (
  pos: number,
  node: ProsemirrorNode<Schema>,
  tr: Transaction<Schema>
) => {
  const $start = tr.doc.resolve(tr.mapping.map(pos));
  const $end = tr.doc.resolve(tr.mapping.map(pos + node.nodeSize));

  return $start.blockRange($end);
};

const createEnsureEntitiesAreWrappedCommand =
  (wrappers?: WrapperNodesList): Command<Schema> =>
  (newState, dispatch) => {
    const { tr, schema } = newState;

    tr.doc.descendants((node, position, parent) => {
      const wrapperNodes = wrappers?.find(([pos]) => position === pos)?.[1];

      /**
       * This position may already be wrapped – due to blocks merging
       */
      if (
        parent.type === schema.nodes.doc &&
        (wrapperNodes || node.type !== schema.nodes.block)
      ) {
        const range = getRangeForNodeAtMappedPosition(position, node, tr);

        if (!range) {
          throw new Error("Cannot rewrap");
        }

        const DEFAULT_WRAPPERS = [
          { type: schema.nodes.block },
          { type: schema.nodes.entity },
        ];

        tr.wrap(
          range,
          wrapperNodes?.map((node) => ({
            type: node.type,
            attrs: node.attrs,
          })) ?? DEFAULT_WRAPPERS
        );
      }

      return false;
    });

    dispatch?.(tr);

    return true;
  };

/**
 * Use to create a copy of the editor state with a certain transaction applied.
 * This is similar to state.apply, but does not take into account plugin
 * appendTransaction (and similar), which can be useful if you need to bypass
 * them.
 */
const stateWithTransaction = (
  state: EditorState<Schema>,
  tr: Transaction<Schema>
) =>
  EditorState.create({
    doc: tr.doc,
    selection: tr.selection,
    plugins: state.plugins,
  });

const combineTransactions = (
  targetTransaction: Transaction<Schema>,
  sourceTransaction: Transaction<Schema>
) => {
  for (const step of sourceTransaction.steps) {
    targetTransaction.step(step);
  }
};

/**
 * This wraps a prosemirror command to unwrap relevant nodes out of their
 * containing block node in order to ensure prosemirror logic that expects text
 * block nodes to be at the top level works as intended. Rewrapping after the
 * prosemirror commands are applied is not handled here, but in a plugin (to
 * ensure that nodes being wrapped by a block is an invariant that can't be
 * accidentally breached)
 *
 * @todo ensure we remove undo item if command fails
 */
const prepareCommandForWrappedEntities =
  (command: Command<Schema>): Command<Schema> =>
  (state, dispatch, view) => {
    // @todo maybe this doesn't work now
    if (state.selection instanceof NodeSelection) {
      return command(state, dispatch, view);
    }
    const { schema, tr } = state;

    // I think this ought to be a map
    const wrappers: WrapperNodesList = [];

    /**
     * First we apply changes to the transaction to unwrap every block
     */
    state.doc.descendants((node, pos) => {
      if ([schema.nodes.block, schema.nodes.entity].includes(node.type)) {
        return true;
      }

      if (node.isTextblock) {
        const range = getRangeForNodeAtMappedPosition(pos, node, tr);

        if (!range) {
          throw new Error("Cannot unwrap");
        }

        const wrapperNodes: ProsemirrorNode<Schema>[] = [];

        const $originalStart = state.doc.resolve(pos);

        for (let depth = $originalStart.depth; depth > 0; depth--) {
          /**
           * The order of wrapperNodes will be the parent order of the
           * replacement wrappers, and as we're traversing up, we need to add
           * to the start of the array
           */
          wrapperNodes.unshift($originalStart.node(depth));
        }

        wrappers.push([pos, wrapperNodes]);
        tr.lift(range, 0);
      }

      return false;
    });

    /**
     * Now that we have a copy of the state with unwrapped blocks, we can run
     * the desired prosemirror command. We pass a custom dispatch function
     * instead of allowing prosemirror to directly dispatch the change to the
     * editor view so that we can capture the transactions generated by
     * prosemirror and merge them into our existing transaction. This allows
     * us to apply all the changes together in one fell swoop, ensuring we
     * don't have awkward intermediary history breakpoints
     *
     * @todo is this sufficient to merge transactions?
     */
    const retVal = command(stateWithTransaction(state, tr), (nextTr) => {
      combineTransactions(tr, nextTr);
    });

    createEnsureEntitiesAreWrappedCommand(
      wrappers.map(([pos, nodes]) => [tr.mapping.map(pos), nodes])
    )(stateWithTransaction(state, tr), (nextTr) => {
      combineTransactions(tr, nextTr);
    });

    dispatch?.(tr);

    return retVal;
  };

const wrapEntitiesKeymap = (baseKeymap: Record<string, Command<Schema>>) =>
  keymap({
    /**
     * Wrap all of the default keymap shortcuts to ensure that the block
     * nodeviews are unwrapped before prosemirror logic is applied (the block
     * nodeview wrappers interfere with this logic)
     */
    ...Object.fromEntries(
      Object.entries(baseKeymap).map(([name, command]) => [
        name,
        prepareCommandForWrappedEntities(command),
      ])
    ),
    // @todo better way of working out that this command doesn't need wrapping
    "Mod-a": baseKeymap["Mod-a"],
  });

export const wrapEntitiesPlugin = (
  baseKeymap: Record<string, Command<Schema>>
) => {
  const wrappedKeymapPlugin = wrapEntitiesKeymap(baseKeymap);

  const ensureEntitiesAreWrappedCommand =
    createEnsureEntitiesAreWrappedCommand();

  /**
   * This plugin ensures at the end of every transaction all necessary nodes
   * are wrapped with block nodeviews
   */
  const ensureWrappedPlugin = new Plugin({
    appendTransaction(transactions, __, newState) {
      let tr;

      ensureEntitiesAreWrappedCommand(newState, (dispatchedTr) => {
        tr = dispatchedTr;
      });

      return tr;
    },
  });

  return [wrappedKeymapPlugin, ensureWrappedPlugin];
};
