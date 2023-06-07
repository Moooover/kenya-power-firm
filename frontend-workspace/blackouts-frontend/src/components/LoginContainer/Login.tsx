/// <reference types="vite-plugin-svgr/client" />

import React from "react";
import { useAuth0 } from "@auth0/auth0-react";
import LoginTwoToneIcon from "@mui/icons-material/LoginTwoTone";
import PersonAddAltOutlinedIcon from "@mui/icons-material/PersonAddAltOutlined";
import { Grid } from "@mui/material";
import { styled } from "@mui/material/styles";
import Container from "@mui/material/Container";
import LoginAppBar from "./Appbar";
import Fab from "@mui/material/Fab";
import HowItWorksCallout from "../../routes/MainPage/HowItWorks";
import Box from "@mui/material/Box";
import Typography from "@mui/material/Typography";

const Login = () => {
  const { loginWithRedirect } = useAuth0();
  return (
    <div>
      <LoginAppBar />
      <Box
        padding={"10px"}
        display={"flex"}
        justifyContent={"center"}
        maxWidth={"700px"}
        margin={"auto"}
      >
        <Typography variant={"subtitle1"}>
          Receive advance notifications days before scheduled power
          interruptions affect your area or nearby areas. Stay informed and be
          prepared.
        </Typography>
      </Box>
      <Box sx={{ marginTop: "10px", padding: "10px" }}>
        <HowItWorksCallout />
      </Box>

      <StyledContainer maxWidth={"sm"}>
        <Grid
          container
          direction="row"
          justifyContent="center"
          alignItems="center"
          spacing={1}
        >
          <StyledGridItem item xs={6}>
            <Fab
              variant="extended"
              color="primary"
              aria-label="login"
              size={"large"}
              onClick={() =>
                loginWithRedirect({
                  authorizationParams: {
                    screen_hint: "login",
                  },
                })
              }
            >
              Login
              <LoginTwoToneIcon sx={{ ml: 1 }} />
            </Fab>
          </StyledGridItem>
          <StyledGridItem item xs={6}>
            <Fab
              variant="extended"
              color="primary"
              aria-label="login"
              size={"large"}
              onClick={() =>
                loginWithRedirect({
                  authorizationParams: {
                    screen_hint: "signup",
                  },
                })
              }
            >
              Sign up
              <PersonAddAltOutlinedIcon sx={{ ml: 1 }} />
            </Fab>
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
