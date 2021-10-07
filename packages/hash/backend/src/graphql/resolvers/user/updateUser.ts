import { ApolloError, ForbiddenError } from "apollo-server-express";

import { Account, User, EntityWithIncompleteEntityType } from "../../../model";
import { MutationUpdateUserArgs, Resolver } from "../../apiTypes.gen";
import { LoggedInGraphQLContext } from "../../context";

export const updateUser: Resolver<
  Promise<EntityWithIncompleteEntityType>,
  {},
  LoggedInGraphQLContext,
  MutationUpdateUserArgs
> = async (_, { userEntityId, properties }, { dataSources, user }) =>
  dataSources.db.transaction(async (client) => {
    /** @todo: allow HASH admins to bypass this */

    if (userEntityId !== user.entityId) {
      throw new ForbiddenError("You can only update your own user properties");
    }

    const { shortname, preferredName, usingHow } = properties;

    if (!shortname && !preferredName && !usingHow) {
      throw new ApolloError(
        "An updated 'shortname', 'preferredName' or 'usingHow' value  must be provided to update a user",
        "NO_OP"
      );
    }

    if (shortname) {
      if (user.properties.shortname === shortname && !preferredName) {
        throw new ApolloError(
          `User with entityId '${user.entityId}' already has the shortname '${shortname}'`,
          "NO_OP"
        );
      }

      await Account.validateShortname(client)(shortname);

      await user.updateShortname(client)(shortname);
    }

    if (preferredName) {
      if (user.properties.preferredName === preferredName) {
        throw new ApolloError(
          `User with entityId '${user.entityId}' already has the preferredName '${preferredName}'`,
          "NO_OP"
        );
      }

      if (!User.preferredNameIsValid(preferredName)) {
        throw new ApolloError(
          `The preferredName '${preferredName}' is invalid`,
          "PREFERRED_NAME_INVALID"
        );
      }

      await user.updatePreferredName(client)(preferredName);
    }

    if (usingHow) {
      if (user.properties.infoProvidedAtSignup.usingHow === usingHow) {
        throw new ApolloError(
          `User with entityId '${user.entityId}' already indicated how they are using HASH '${usingHow}'`,
          "NO_OP"
        );
      }

      await user.updateInfoProvidedAtSignup(client)({ usingHow });
    }

    return user.toGQLUnknownEntity();
  });
