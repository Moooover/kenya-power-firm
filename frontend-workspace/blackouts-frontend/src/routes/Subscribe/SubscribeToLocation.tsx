import { Input } from "@mui/material";
import SearchBox from "../../components/SearchBox";
import useSWR from "swr";
import React from "react";

import SearchForLocation, {
  LocationSearchData,
} from "./components/SearchForLocation";
import DrawerOnSelectLocation from "./components/DrawerOnSelectLocation";

export default function SubscribeToLocation() {
  const [openConfirmation, setOpenConfirmation] = React.useState(false);
  const [locationSelected, setLocationSelected] =
    React.useState<LocationSearchData | null>(null);
  const onSelectLocation = (location: LocationSearchData) => {
    setLocationSelected(location);
    setDrawerState(true);
  };
  const setDrawerState = (value: boolean) => {
    setOpenConfirmation(value);
  };
  return (
    <div>
      <SearchForLocation onSelectLocation={onSelectLocation} />
      {locationSelected && (
        <DrawerOnSelectLocation
          drawerState={openConfirmation}
          setDrawerState={setDrawerState}
          location={locationSelected}
        />
      )}
    </div>
  );
}
