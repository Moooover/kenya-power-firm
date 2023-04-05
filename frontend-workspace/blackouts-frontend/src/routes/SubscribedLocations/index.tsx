import { AxiosError } from "axios";
import useSWR, { mutate } from "swr";
import * as React from "react";
import ListSubheader from "@mui/material/ListSubheader";
import List from "@mui/material/List";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import Collapse from "@mui/material/Collapse";
import InboxIcon from "@mui/icons-material/MoveToInbox";
import DraftsIcon from "@mui/icons-material/Drafts";
import SendIcon from "@mui/icons-material/Send";
import ExpandLess from "@mui/icons-material/ExpandLess";
import ExpandMore from "@mui/icons-material/ExpandMore";
import StarBorder from "@mui/icons-material/StarBorder";
import MyLocationTwoToneIcon from "@mui/icons-material/MyLocationTwoTone";
import DeleteTwoToneIcon from "@mui/icons-material/DeleteTwoTone";
import NearMeTwoToneIcon from "@mui/icons-material/NearMeTwoTone";
import { useDeleteLocationSubscription } from "../../hooks/mutations/useDeleteSubscribedLocation";

type AdjuscentLocation = {
  id: string;
  name: string;
  address: string;
};

type Location = {
  id: string;
  name: string;
  address: string;
  adjuscent_locations: AdjuscentLocation[];
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
              <Location location={location} />
            ))}
          </div>
        </List>
      )}
    </div>
  );
}

function Location({ location }: { location: Location }) {
  const [open, setOpen] = React.useState(false);

  const { trigger } = useDeleteLocationSubscription(location.id);

  const handleToggleAdjuscentLocations = () => {
    setOpen(!open);
  };
  const handleDeleteSubscribed = async () => {
    await trigger();
    mutate("/locations/list/subscribed");
  };
  return (
    <>
      <ListItemButton disableRipple>
        <ListItemIcon>
          <MyLocationTwoToneIcon />
        </ListItemIcon>
        <ListItemText
          primary={location.name}
          secondary={location.address}
          onClick={handleToggleAdjuscentLocations}
        />
        <ListItemIcon onClick={handleDeleteSubscribed}>
          <DeleteTwoToneIcon />
        </ListItemIcon>
        {location.adjuscent_locations.length > 0 ? (
          <div>
            {open ? (
              <ExpandLess onClick={handleToggleAdjuscentLocations} />
            ) : (
              <ExpandMore onClick={handleToggleAdjuscentLocations} />
            )}
          </div>
        ) : (
          <ExpandLess
            style={{
              color: "transparent",
            }}
          />
        )}
      </ListItemButton>
      {location && (
        <Collapse in={open} timeout="auto" unmountOnExit>
          <List component="div" disablePadding>
            {location.adjuscent_locations.map((adjuscentLocation) => {
              return (
                <ListItemButton
                  disableRipple
                  sx={{ pl: 10 }}
                  key={adjuscentLocation.id}
                >
                  <ListItemIcon>
                    <NearMeTwoToneIcon />
                  </ListItemIcon>
                  <ListItemText
                    primary={adjuscentLocation.name}
                    secondary={adjuscentLocation.address}
                  />
                </ListItemButton>
              );
            })}
          </List>
        </Collapse>
      )}
    </>
  );
}
