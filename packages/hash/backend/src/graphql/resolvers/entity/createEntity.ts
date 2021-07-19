import { genId } from "../../../util";
import { DbUnknownEntity } from "../../../types/dbTypes";
import {
  MutationCreateEntityArgs,
  Resolver,
  Visibility,
} from "../../autoGeneratedTypes";
import { GraphQLContext } from "../../context";

export const createEntity: Resolver<
  Promise<DbUnknownEntity>,
  {},
  GraphQLContext,
  MutationCreateEntityArgs
> = async (_, { accountId, properties, type }, { dataSources }) => {
  const dbEntity = await dataSources.db.createEntity({
    accountId,
    createdById: genId(), // TODO
    type,
    properties,
  });

  const entity: DbUnknownEntity = {
    ...dbEntity,
    id: dbEntity.entityId,
    accountId: dbEntity.accountId,
    visibility: Visibility.Public, // TODO: should be a param?,
  };

  return entity;
};
