import { ApolloQueryResult, QueryHookOptions, useQuery } from "@apollo/client";
import * as Sentry from "@sentry/nextjs";
import { useRouter } from "next/router";
import { useContext, useEffect, useLayoutEffect, useMemo, useRef } from "react";

import { Subgraph, SubgraphRootTypes } from "@hashintel/hash-subgraph";
import { meQuery } from "../../graphql/queries/user.queries";
import { MeQuery, MeQueryVariables } from "../../graphql/apiTypes.gen";
import { oryKratosClient } from "../../pages/shared/ory-kratos";
import { AuthenticatedUser, constructAuthenticatedUser } from "../../lib/user";
import { useInitTypeSystem } from "../../lib/use-init-type-system";
import { SessionContext } from "../../pages/shared/session-context";

/**
 * Returns an object containing:
 *
 * user: the authenticated user (if any)
 *
 * refetch: a function to refetch the user from the API (ApolloClient will update the cache with the return)
 *
 * loading: a boolean to check if the api call is still loading
 */
export const useAuthenticatedUser = (
  options?: Omit<QueryHookOptions<MeQuery, MeQueryVariables>, "errorPolicy">,
  forceLogin = false,
) => {
  const loadingTypeSystem = useInitTypeSystem();
  const router = useRouter();

  const {
    data: meQueryResponseData,
    refetch: refetchUser,
    loading: loadingUser,
  } = useQuery<MeQuery, MeQueryVariables>(meQuery, {
    ...options,
    errorPolicy: "all",
  });

  const { me: subgraph } = meQueryResponseData ?? {};

  const {
    kratosSession,
    loadingKratosSession,
    setKratosSession,
    setLoadingKratosSession,
  } = useContext(SessionContext);

  const authenticatedUser = useMemo<AuthenticatedUser | undefined>(
    () =>
      !loadingTypeSystem && subgraph && kratosSession
        ? constructAuthenticatedUser({
            userEntityEditionId: (
              subgraph as Subgraph<SubgraphRootTypes["entity"]>
            ).roots[0]!,
            /**
             * @todo: ensure this subgraph contains the incoming links of orgs
             * at depth 2 to support constructing the `members` of an `Org`.
             *
             * @see https://app.asana.com/0/1202805690238892/1203250435416412/f
             */
            subgraph,
            kratosSession,
          })
        : undefined,
    [subgraph, kratosSession, loadingTypeSystem],
  );

  useEffect(() => {
    Sentry.configureScope((scope) => {
      const sentryUser = scope.getUser();
      if (!authenticatedUser && sentryUser) {
        scope.setUser(null);
      } else if (
        authenticatedUser &&
        sentryUser?.id !== authenticatedUser.entityEditionId.baseId
      ) {
        const primaryEmail = authenticatedUser.emails.find(
          (email) => email.primary,
        );
        Sentry.setUser({
          id: authenticatedUser.entityEditionId.baseId,
          email: primaryEmail?.address,
        });
      }
    });
  }, [authenticatedUser]);

  const fetchKratosIdentity = async (login: boolean) => {
    setLoadingKratosSession(true);

    const session = await oryKratosClient
      .toSession()
      .then(({ data }) => data)
      .catch(() => undefined);

    if (!session && login) {
      await router.push("/login");
    } else {
      setKratosSession(session);
      setLoadingKratosSession(false);
      return session;
    }
  };

  const fetchKratosIdentityRef = useRef(fetchKratosIdentity);
  useLayoutEffect(() => {
    fetchKratosIdentityRef.current = fetchKratosIdentity;
  });

  useEffect(() => {
    void fetchKratosIdentityRef.current(forceLogin);
  }, [forceLogin]);

  /**
   * @todo add method to manually update user cache after
   * a query/mutation returns the updated User object.
   * This will help prevent having to call meQuery after every mutation
   */
  // const updateCache = () => {
  //   client.writeQuery();
  // };

  return {
    authenticatedUser,
    kratosSession,
    refetch: () =>
      Promise.all([
        // Until we change the GraphQL query scalars to return constrained Subgraphs, we need to (safely) cast
        // ourselves
        (
          refetchUser as () => Promise<
            ApolloQueryResult<{ me: Subgraph<SubgraphRootTypes["entity"]> }>
          >
        )(),
        fetchKratosIdentity(forceLogin),
      ]),
    loading: loadingUser || loadingKratosSession,
  };
};

export const useLoggedInUser = (
  options?: Parameters<typeof useAuthenticatedUser>[0],
) => {
  return useAuthenticatedUser(options, true);
};
