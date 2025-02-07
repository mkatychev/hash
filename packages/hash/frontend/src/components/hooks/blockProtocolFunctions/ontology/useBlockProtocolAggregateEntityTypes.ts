import { useLazyQuery } from "@apollo/client";

import { useCallback } from "react";
import { Subgraph, SubgraphRootTypes } from "@hashintel/hash-subgraph";
import {
  GetAllLatestEntityTypesQuery,
  GetAllLatestEntityTypesQueryVariables,
} from "../../../../graphql/apiTypes.gen";
import { getAllLatestEntityTypesQuery } from "../../../../graphql/queries/ontology/entity-type.queries";
import { AggregateEntityTypesMessageCallback } from "./ontology-types-shim";

export const useBlockProtocolAggregateEntityTypes = (): {
  aggregateEntityTypes: AggregateEntityTypesMessageCallback;
} => {
  const [aggregateFn] = useLazyQuery<
    GetAllLatestEntityTypesQuery,
    GetAllLatestEntityTypesQueryVariables
  >(getAllLatestEntityTypesQuery, {
    /** @todo reconsider caching. This is done for testing/demo purposes. */
    fetchPolicy: "no-cache",
  });

  const aggregateEntityTypes = useCallback<AggregateEntityTypesMessageCallback>(
    async ({ data }) => {
      if (!data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "'data' must be provided for aggregateEntityTypes",
            },
          ],
        };
      }

      /**
       * @todo Add filtering to this aggregate query using structural querying.
       *   This may mean having the backend use structural querying and relaying
       *   or doing it from here.
       *   https://app.asana.com/0/1202805690238892/1202890614880643/f
       */
      const response = await aggregateFn({
        variables: {
          dataTypeResolveDepth: 255,
          propertyTypeResolveDepth: 255,
          entityTypeResolveDepth: 0,
        },
      });

      if (!response.data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "Error calling aggregateEntityTypes",
            },
          ],
        };
      }

      return {
        /** @todo - Is there a way we can ergonomically encode this in the GraphQL type? */
        data: response.data.getAllLatestEntityTypes as Subgraph<
          SubgraphRootTypes["entityType"]
        >,
      };
    },
    [aggregateFn],
  );

  return { aggregateEntityTypes };
};
