import * as React from "react";
import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import CssBaseline from "@mui/material/CssBaseline";
import IconButton from "@mui/material/IconButton";
import Button from "@mui/material/Button";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { useAuth0 } from "@auth0/auth0-react";
import EmojiObjectsTwoToneIcon from "@mui/icons-material/EmojiObjectsTwoTone";

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
            <EmojiObjectsTwoToneIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div" sx={{ flexGrow: 1 }}>
            KPLC Alerts (Beta)
          </Typography>

          <Button
            color="inherit"
            onClick={() =>
              logout({
                logoutParams: {
                  returnTo: window.location.origin,
                },
              })
            }
          >
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
