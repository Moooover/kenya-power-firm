import * as React from "react";
import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import CssBaseline from "@mui/material/CssBaseline";
import IconButton from "@mui/material/IconButton";
import Button from "@mui/material/Button";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { useAuth0 } from "@auth0/auth0-react";
import LightIcon from "@mui/icons-material/Light";

type Props = {};
export default function Layout(props: React.PropsWithChildren<Props>) {
  const { children } = props;
  const { logout } = useAuth0();

  return (
    <Box sx={{ display: "flex" }}>
      <CssBaseline />
      <AppBar position="fixed">
        <Toolbar>
          <IconButton aria-label="bulb" color="secondary">
            <LightIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            Blackout Alerts
          </Typography>

          <Button color="inherit" onClick={() => logout()}>
            LogOut
          </Button>
        </Toolbar>
      </AppBar>

      <Box
        component="main"
        sx={{
          flexGrow: 1,
          p: 3,
        }}
      >
        <Toolbar />
        {children}
      </Box>
    </Box>
  );
}
