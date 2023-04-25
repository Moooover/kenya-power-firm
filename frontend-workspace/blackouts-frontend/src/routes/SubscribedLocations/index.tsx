import { AxiosError } from "axios";
import useSWR, { mutate } from "swr";
import * as React from "react";
import ListSubheader from "@mui/material/ListSubheader";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Collapse from "@mui/material/Collapse";

import ExpandLess from "@mui/icons-material/ExpandLess";
import ExpandMore from "@mui/icons-material/ExpandMore";

import DeleteTwoToneIcon from "@mui/icons-material/DeleteTwoTone";
import NearMeTwoToneIcon from "@mui/icons-material/NearMeTwoTone";
import { useDeleteLocationSubscription } from "./useDeleteSubscribedLocation";
import Avatar from "@mui/material/Avatar";
import UnsubscribeDialog from "./UnsubscribeDialog";

type Location = {
  id: string;
  name: string;
  address: string;
};
type Response = {
  items: Location[];
};
function useGetSubscribedLocations() {
  return useSWR<Response, AxiosError>("/locations/list/subscribed");
}

export default function SubscribedLocations() {
  const { data } = useGetSubscribedLocations();
  return (
    <div>
      {data && (
        <List
          sx={{ width: "100%", bgcolor: "background.paper" }}
          component="nav"
          aria-labelledby="nested-list-subheader"
          subheader={
            <ListSubheader component="div" id="nested-list-subheader">
              Subscribed Locations
            </ListSubheader>
          }
        >
          <div>
            {data.items.map((location) => (
              <Location key={location.id} location={location} />
            ))}
          </div>
        </List>
      )}
    </div>
  );
}

function Location({ location }: { location: Location }) {
  const [open, setOpen] = React.useState(false);

  const [openDialog, setOpenDialog] = React.useState(false);

  const handleToggleAdjuscentLocations = () => {
    setOpen(!open);
  };

  const handleCloseAlertDialog = () => {
    setOpenDialog(false);
  };
  return (
    <>
      <ListItemButton disableRipple>
        <ListItemIcon>
          <Avatar>{location.name[0]}</Avatar>
        </ListItemIcon>
        <ListItemText
          primary={location.name}
          secondary={location.address}
          onClick={handleToggleAdjuscentLocations}
        />
        <ListItemIcon onClick={() => setOpenDialog(true)}>
          <DeleteTwoToneIcon />
        </ListItemIcon>
      </ListItemButton>
      <UnsubscribeDialog
        open={openDialog}
        location={location}
        closeDialog={handleCloseAlertDialog}
      />
    </>
  );
}
