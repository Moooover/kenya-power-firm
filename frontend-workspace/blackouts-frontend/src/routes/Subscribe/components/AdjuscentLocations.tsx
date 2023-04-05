import * as React from "react";
import { styled } from "@mui/material/styles";
import Chip from "@mui/material/Chip";
import Paper from "@mui/material/Paper";
import TagFacesIcon from "@mui/icons-material/TagFaces";
import { LocationSearchData } from "./SearchForLocation";

interface ChipData {
  key: number;
  label: string;
}

const ListItem = styled("li")(({ theme }) => ({
  margin: theme.spacing(0.5),
}));

type Props = {
  locations: LocationSearchData[];
  handleDelete: (data: LocationSearchData) => () => void;
};

export default function AdjuscentLocations({ locations, handleDelete }: Props) {
  return (
    <Paper
      sx={{
        display: "flex",
        justifyContent: "center",
        flexWrap: "wrap",
        listStyle: "none",
        p: 0.5,
        m: 0,
      }}
      elevation={10}
      component="ul"
    >
      {locations.map((data) => {
        return (
          <ListItem key={data.id}>
            <Chip label={data.name} onDelete={handleDelete(data)} />
          </ListItem>
        );
      })}
    </Paper>
  );
}
