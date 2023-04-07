import * as React from "react";
import List from "@mui/material/List";
import Divider from "@mui/material/Divider";
import ListItemText from "@mui/material/ListItemText";
import ListItemAvatar from "@mui/material/ListItemAvatar";
import Avatar from "@mui/material/Avatar";
import Typography from "@mui/material/Typography";
import { LocationSearchData } from "./SearchForLocation";
import { ListItemButton } from "@mui/material";
import ListItemIcon from "@mui/material/ListItemIcon";

type Props = {
  items: LocationSearchData[];
  onClick: (item: LocationSearchData) => void;
};

export default function ListOfLocations(props: Props) {
  return (
    <List
      sx={{ width: "100%", bgcolor: "background.paper", marginTop: "10px" }}
    >
      {props.items.map((item, idx) => {
        return (
          <>
            <ListItemButton
              alignItems="flex-start"
              key={item.id}
              onClick={() => {
                props.onClick(item);
              }}
            >
              <ListItemAvatar>
                <ListItemIcon>
                  <Avatar>{item.name[0]}</Avatar>
                </ListItemIcon>
              </ListItemAvatar>
              <ListItemText
                primary={item.name}
                secondary={
                  <React.Fragment>
                    <Typography
                      sx={{ display: "inline" }}
                      component="span"
                      variant="body2"
                      color="text.primary"
                    >
                      {item.address}
                    </Typography>
                  </React.Fragment>
                }
              />
            </ListItemButton>
            <Divider variant="inset" component="li" key={item.id + idx} />
          </>
        );
      })}
    </List>
  );
}
