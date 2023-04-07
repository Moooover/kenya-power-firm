import * as React from "react";
import { LocationSearchData } from "./SearchForLocation";
import List from "@mui/material/List";
import Divider from "@mui/material/Divider";
import ListItemText from "@mui/material/ListItemText";
import Avatar from "@mui/material/Avatar";
import Typography from "@mui/material/Typography";
import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import RemoveIcon from "@mui/icons-material/Remove";
import IconButton from "@mui/material/IconButton";
import ListSubheader from "@mui/material/ListSubheader";
interface ChipData {
  key: number;
  label: string;
}

type Props = {
  locations: LocationSearchData[];
  handleDelete: (data: LocationSearchData) => () => void;
};

export default function AdjuscentLocations({ locations, handleDelete }: Props) {
  return (
    <List
      subheader={<ListSubheader>Adjuscent Locations</ListSubheader>}
      sx={{ width: "100%", bgcolor: "background.paper" }}
    >
      {locations.map((data) => {
        return (
          <>
            <ListItem
              alignItems="flex-start"
              key={data.id}
              secondaryAction={
                <IconButton onClick={handleDelete(data)}>
                  <RemoveIcon />
                </IconButton>
              }
            >
              <ListItemIcon>
                <Avatar>{data.name[0]}</Avatar>
              </ListItemIcon>
              <ListItemText
                primary={data.name}
                secondary={
                  <React.Fragment>
                    <Typography
                      sx={{ display: "inline" }}
                      component="span"
                      variant="body2"
                      color="text.primary"
                    >
                      {data.address}
                    </Typography>
                  </React.Fragment>
                }
              />
            </ListItem>
            <Divider
              variant="inset"
              component="li"
              key={`${data.id}-divider`}
            />
          </>
        );
      })}
    </List>
  );
}
