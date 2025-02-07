import { useEffect } from "react";
import { useRouter } from "next/router";

import { Box, Container, Typography } from "@mui/material";
import { useLogoutFlow } from "../components/hooks/useLogoutFlow";
import { useLoggedInUser } from "../components/hooks/useAuthenticatedUser";
import { NextPageWithLayout } from "../shared/layout";
import { Button } from "../shared/ui";

const Page: NextPageWithLayout = () => {
  const router = useRouter();
  const { authenticatedUser, kratosSession } = useLoggedInUser();

  const { logout } = useLogoutFlow();

  useEffect(() => {
    if (authenticatedUser) {
      void router.push(`/${authenticatedUser.userAccountId}`);
    }
  }, [router, authenticatedUser]);

  /** @todo: remove session developer information */
  return (
    <Container sx={{ pt: 10 }}>
      {authenticatedUser && (
        <Typography variant="h1">
          Hi {authenticatedUser.preferredName}!
        </Typography>
      )}
      <Typography variant="h2" gutterBottom>
        {kratosSession
          ? "You have a kratos session"
          : "You don't have a kratos session"}
      </Typography>
      <Button onClick={logout}>Log out</Button>
      <Typography gutterBottom>
        This is what your kratos session looks like:
      </Typography>
      {kratosSession ? (
        <Box component="pre">{JSON.stringify(kratosSession, null, 2)}</Box>
      ) : null}
    </Container>
  );
};

export default Page;
