import ListItem from "@mui/material/ListItem";
import ListItemIcon from "@mui/material/ListItemIcon";
import { Skeleton } from "@mui/lab";
import Avatar from "@mui/material/Avatar";
import ListItemText from "@mui/material/ListItemText";
import * as React from "react";

export default function Loading() {
  const arr = [1, 2, 3];
  return (
    <div>
      {arr.map((i) => (
        <ListItem key={i}>
          <ListItemIcon>
            <Skeleton variant={"circular"}>
              <Avatar />
            </Skeleton>
          </ListItemIcon>
          <ListItemText>
            <Skeleton variant={"text"} width={"60%"} />
            <Skeleton variant={"text"} />
          </ListItemText>
        </ListItem>
      ))}
    </div>
  );
}
