import { useLazyQuery } from "@apollo/client";

import { useCallback } from "react";
import { Subgraph, SubgraphRootTypes } from "@hashintel/hash-subgraph";
import {
  GetEntityTypeQuery,
  GetEntityTypeQueryVariables,
} from "../../../../graphql/apiTypes.gen";
import { getEntityTypeQuery } from "../../../../graphql/queries/ontology/entity-type.queries";
import { GetEntityTypeMessageCallback } from "./ontology-types-shim";

export const useBlockProtocolGetEntityType = (): {
  getEntityType: GetEntityTypeMessageCallback;
} => {
  const [getFn] = useLazyQuery<GetEntityTypeQuery, GetEntityTypeQueryVariables>(
    getEntityTypeQuery,
    {
      /**
       * Entity types are immutable, any request for an entityTypeId should always return the same value.
       * However, currently requests for non-existent entity types currently return an empty subgraph, so
       * we can't rely on this.
       *
       * @todo revert this back to cache-first once that's fixed
       */
      fetchPolicy: "network-only",
    },
  );

  const getEntityType = useCallback<GetEntityTypeMessageCallback>(
    async ({ data: entityTypeId }) => {
      if (!entityTypeId) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "'data' must be provided for getEntityType",
            },
          ],
        };
      }

      const response = await getFn({
        query: getEntityTypeQuery,
        variables: {
          entityTypeId,
          dataTypeResolveDepth: 255,
          propertyTypeResolveDepth: 255,
          entityTypeResolveDepth: 1,
        },
      });

      if (!response.data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "Error calling getEntityType",
            },
          ],
        };
      }

      return {
        /** @todo - Is there a way we can ergonomically encode this in the GraphQL type? */
        data: response.data.getEntityType as Subgraph<
          SubgraphRootTypes["entityType"]
        >,
      };
    },
    [getFn],
  );

  return { getEntityType };
};
