import { isPlainObject } from "lodash";
import { isUnknownObject } from "@hashintel/hash-shared/util";
import { Subgraph, SubgraphRootTypes } from "@hashintel/hash-subgraph";
import { getPropertyTypesByBaseUri } from "@hashintel/hash-subgraph/src/stdlib/element/property-type";
import { PropertyRow } from "../../types";
import { getExpectedTypesOfPropertyType } from "./get-expected-types-of-property-type";

/**
 * This function generates property row data,
 * and calls itself again for each nested property. Then puts results of these recursive calls into `children` array
 *
 * @param properties
 * Properties object for current depth. On entity level, it starts from `entity.properties`
 *
 * @param propertyTypeBaseUri
 * Name of the specific property inside `properties` object
 *
 * @param propertyKeyChain
 * An array of `propertyTypeBaseUri`'s to store the path to each nested property.
 * ```js
 * // Example: `propertyKeyChain` of `c`
 * properties = { a: { b: { c: "John" } } };
 * propertyKeyChain = ["a", "b", "c"]
 * ```
 *
 * @param entitySubgraph
 * An object storing root entity & subgraph of that entity
 *
 * @param requiredPropertyTypes
 * An array of `propertyTypeBaseUri`'s.
 * This is used to check if a property should be shown as `required` or not
 *
 * @param depth
 * Depth of the property. For properties at root, depth starts from `0`
 *
 * @returns property row (and nested rows as `children` if it's a nested property)
 */
export const generatePropertyRowRecursively = (
  properties: unknown,
  propertyTypeBaseUri: string,
  propertyKeyChain: string[],
  entitySubgraph: Subgraph<SubgraphRootTypes["entity"]>,
  requiredPropertyTypes: string[],
  depth = 0,
): PropertyRow => {
  const propertyTypeVersions = getPropertyTypesByBaseUri(
    entitySubgraph,
    propertyTypeBaseUri,
  );

  if (!propertyTypeVersions) {
    throw new Error(
      `propertyType not found for base URI: ${propertyTypeBaseUri}`,
    );
  }

  const propertyType = propertyTypeVersions[0]!.schema;

  const { isArray, expectedTypes } = getExpectedTypesOfPropertyType(
    propertyType,
    entitySubgraph,
  );

  const required = !!requiredPropertyTypes?.includes(propertyTypeBaseUri);

  let value;
  const children: PropertyRow[] = [];

  if (isUnknownObject(properties)) {
    value = properties[propertyTypeBaseUri];

    // generate rows for nested properties and push them to children array
    if (isPlainObject(value) && isUnknownObject(value)) {
      for (const subPropertyTypeBaseUri of Object.keys(value)) {
        children.push(
          generatePropertyRowRecursively(
            properties[propertyTypeBaseUri],
            subPropertyTypeBaseUri,
            [...propertyKeyChain, subPropertyTypeBaseUri],
            entitySubgraph,
            requiredPropertyTypes,
            depth + 1,
          ),
        );
      }
    }
  }

  const indent = !depth ? 0 : children.length ? depth : depth - 1;

  const rowId = propertyKeyChain.join(".");

  return {
    rowId,
    title: propertyType.title,
    value,
    expectedTypes,
    isArray,
    required,
    depth,
    children,
    indent,
    /**
     * this will be filled by `fillRowDataIndentCalculations`
     * this is not filled here, because we'll use the whole flattened tree,
     * and check some values of prev-next items on the flattened tree while calculating this
     */
    verticalLinesForEachIndent: [],
    propertyKeyChain,
  };
};
