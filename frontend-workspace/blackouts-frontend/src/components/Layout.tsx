import * as React from "react";
import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import CssBaseline from "@mui/material/CssBaseline";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import IconButton from "@mui/material/IconButton";
import InboxIcon from "@mui/icons-material/MoveToInbox";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import MailIcon from "@mui/icons-material/Mail";
import MenuIcon from "@mui/icons-material/Menu";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import LogoutTwoToneIcon from "@mui/icons-material/LogoutTwoTone";
import { useAuth0 } from "@auth0/auth0-react";
import LightIcon from "@mui/icons-material/Light";
import AddLocationAltTwoToneIcon from "@mui/icons-material/AddLocationAltTwoTone";
import ListAltTwoToneIcon from "@mui/icons-material/ListAltTwoTone";
import { redirect, useLocation, useNavigate } from "react-router-dom";

const drawerWidth = 300;

interface Props {
  /**
   * Injected by the documentation to work in an iframe.
   * You won't need it on your project.
   */
  window?: () => Window;
}

const listItemButtonStyle = {
  borderRadius: "10px",
  ml: 2,
  mr: 2,
  mb: 2,
  alignItems: "flex-start",
};

const DrawerItems = () => {
  const { logout } = useAuth0();
  const navigate = useNavigate();
  const { pathname } = useLocation();

  return (
    <div>
      <Toolbar />
      <Divider />
      <List>
        <ListItem disablePadding>
          <ListItemButton
            selected={pathname === "/"}
            sx={listItemButtonStyle}
            onClick={() => {
              navigate("/");
            }}
          >
            <ListItemIcon sx={{ my: "auto" }}>
              <AddLocationAltTwoToneIcon />
            </ListItemIcon>
            <ListItemText primary={"Subscribe"} />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton
            selected={pathname === "/subscribed"}
            sx={listItemButtonStyle}
            onClick={() => {
              navigate("/subscribed");
            }}
          >
            <ListItemIcon sx={{ my: "auto" }}>
              <ListAltTwoToneIcon />
            </ListItemIcon>
            <ListItemText primary={"View subscribed"} />
          </ListItemButton>
        </ListItem>
        <Divider />

        <ListItem disablePadding>
          <ListItemButton sx={listItemButtonStyle} onClick={() => logout()}>
            <ListItemIcon sx={{ my: "auto" }}>
              <LogoutTwoToneIcon />
            </ListItemIcon>
            <ListItemText primary={"Logout"} />
          </ListItemButton>
        </ListItem>
      </List>
    </div>
  );
};

export default function Layout(props: React.PropsWithChildren<Props>) {
  const { window, children } = props;
  const [mobileOpen, setMobileOpen] = React.useState(false);

  const handleDrawerToggle = () => {
    setMobileOpen(!mobileOpen);
  };

  let location = useLocation();

  React.useEffect(() => {
    if (mobileOpen) setMobileOpen(false);
  }, [location]);

  const container =
    window !== undefined ? () => window().document.body : undefined;

  return (
    <Box sx={{ display: "flex" }}>
      <CssBaseline />
      <AppBar
        position="fixed"
        sx={{
          width: { sm: `calc(100% - ${drawerWidth}px)` },
          ml: { sm: `${drawerWidth}px` },
        }}
      >
        <Toolbar>
          <IconButton
            color="inherit"
            aria-label="open drawer"
            edge="start"
            onClick={handleDrawerToggle}
            sx={{ mr: 2, display: { sm: "none" } }}
          >
            <MenuIcon />
          </IconButton>
          <Typography variant="h6" noWrap component="div">
            Blackout
          </Typography>
          <IconButton aria-label="bulb" color="secondary">
            <LightIcon />
          </IconButton>
        </Toolbar>
      </AppBar>
      <Box
        component="nav"
        sx={{ width: { sm: drawerWidth }, flexShrink: { sm: 0 } }}
        aria-label="mailbox folders"
      >
        {/* The implementation can be swapped with js to avoid SEO duplication of links. */}
        <Drawer
          container={container}
          variant="temporary"
          open={mobileOpen}
          onClose={handleDrawerToggle}
          ModalProps={{
            keepMounted: true, // Better open performance on mobile.
          }}
          sx={{
            display: { xs: "block", sm: "none" },
            "& .MuiDrawer-paper": {
              boxSizing: "border-box",
              width: drawerWidth,
            },
          }}
        >
          <DrawerItems />
        </Drawer>
        <Drawer
          variant="permanent"
          sx={{
            display: { xs: "none", sm: "block" },
            "& .MuiDrawer-paper": {
              boxSizing: "border-box",
              width: drawerWidth,
            },
          }}
          open
        >
          <DrawerItems />
        </Drawer>
      </Box>
      <Box
        component="main"
        sx={{
          flexGrow: 1,
          p: 3,
          width: { sm: `calc(100% - ${drawerWidth}px)` },
        }}
      >
        <Toolbar />
        {children}
      </Box>
    </Box>
  );
}
