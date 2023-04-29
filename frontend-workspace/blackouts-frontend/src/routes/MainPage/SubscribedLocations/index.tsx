import { AxiosError } from "axios";
import useSWR from "swr";
import * as React from "react";
import ListSubheader from "@mui/material/ListSubheader";
import List from "@mui/material/List";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";

import DeleteTwoToneIcon from "@mui/icons-material/DeleteTwoTone";
import Avatar from "@mui/material/Avatar";
import UnsubscribeDialog from "./UnsubscribeDialog";
import Loading from "../LoadingLocations";
import ListItem from "@mui/material/ListItem";

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
  const { data, isLoading } = useGetSubscribedLocations();
  return (
    <div>
      {isLoading && <Loading />}
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
      <ListItem>
        <ListItemIcon>
          <Avatar>{location.name[0]}</Avatar>
        </ListItemIcon>
        <ListItemText
          primary={location.name}
          secondary={location.address}
          onClick={handleToggleAdjuscentLocations}
        />
        <ListItemIcon
          sx={{
            cursor: "pointer",
          }}
          onClick={() => setOpenDialog(true)}
        >
          <DeleteTwoToneIcon />
        </ListItemIcon>
      </ListItem>
      <UnsubscribeDialog
        open={openDialog}
        location={location}
        closeDialog={handleCloseAlertDialog}
      />
    </>
  );
}
