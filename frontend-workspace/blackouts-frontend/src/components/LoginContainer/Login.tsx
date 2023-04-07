/// <reference types="vite-plugin-svgr/client" />

import React from "react";
import { useAuth0 } from "@auth0/auth0-react";
import LoginTwoToneIcon from "@mui/icons-material/LoginTwoTone";
import { Grid } from "@mui/material";
import { ReactComponent as LeftDoodle } from "./LeftDoodle.svg";
import { ReactComponent as RightDoodle } from "./RightDoodle.svg";
import { styled } from "@mui/material/styles";
import Container from "@mui/material/Container";
import LoginAppBar from "./Appbar";
import Fab from "@mui/material/Fab";

const Login = () => {
  const { loginWithRedirect } = useAuth0();
  return (
    <div>
      <LoginAppBar />
      <StyledContainer maxWidth={"sm"}>
        <Grid
          container
          direction="row"
          justifyContent="center"
          alignItems="center"
        >
          <StyledGridItem item xs={3}>
            <LeftDoodle />
          </StyledGridItem>
          <StyledGridItem item xs={6}>
            <Fab
              variant="extended"
              color="primary"
              aria-label="login"
              size={"large"}
              onClick={() => loginWithRedirect()}
            >
              Login
              <LoginTwoToneIcon sx={{ ml: 1 }} />
            </Fab>
          </StyledGridItem>
          <StyledGridItem item xs={3}>
            <RightDoodle />
          </StyledGridItem>
        </Grid>
      </StyledContainer>
    </div>
  );
};

const StyledGridItem = styled(Grid)({
  display: "flex",
  justifyContent: "center",
});

const StyledContainer = styled(Container)({
  position: "fixed",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
});

export default Login;
