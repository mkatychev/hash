import { useLazyQuery } from "@apollo/client";

import { useCallback } from "react";
import { Subgraph, SubgraphRootTypes } from "@hashintel/hash-subgraph";
import {
  GetDataTypeQuery,
  GetDataTypeQueryVariables,
} from "../../../../graphql/apiTypes.gen";
import { getDataTypeQuery } from "../../../../graphql/queries/ontology/data-type.queries";
import { GetDataTypeMessageCallback } from "./ontology-types-shim";

export const useBlockProtocolGetDataType = (): {
  getDataType: GetDataTypeMessageCallback;
} => {
  const [getFn] = useLazyQuery<GetDataTypeQuery, GetDataTypeQueryVariables>(
    getDataTypeQuery,
    {
      // Data types are immutable, any request for an dataTypeId should always return the same value.
      fetchPolicy: "cache-first",
    },
  );

  const getDataType = useCallback<GetDataTypeMessageCallback>(
    async ({ data: dataTypeId }) => {
      if (!dataTypeId) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "'data' must be provided for getDataType",
            },
          ],
        };
      }

      const response = await getFn({
        variables: {
          dataTypeId,
          dataTypeResolveDepth: 255,
        },
      });

      if (!response.data) {
        return {
          errors: [
            {
              code: "INVALID_INPUT",
              message: "Error calling getDataType",
            },
          ],
        };
      }

      return {
        /** @todo - Is there a way we can ergonomically encode this in the GraphQL type? */
        data: response.data.getDataType as Subgraph<
          SubgraphRootTypes["dataType"]
        >,
      };
    },
    [getFn],
  );

  return { getDataType };
};
