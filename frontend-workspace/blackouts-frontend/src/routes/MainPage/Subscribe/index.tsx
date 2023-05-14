import { Input } from "@mui/material";
import SearchBox from "../../../components/SearchBox";
import useSWR, { mutate } from "swr";
import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import PhoneIcon from "@mui/icons-material/Phone";
import Grid from "@mui/material/Grid";
import FavoriteIcon from "@mui/icons-material/Favorite";
import PersonPinIcon from "@mui/icons-material/PersonPin";
import PhoneMissedIcon from "@mui/icons-material/PhoneMissed";
import MyLocationTwoToneIcon from "@mui/icons-material/MyLocationTwoTone";
import AddLocationAltTwoToneIcon from "@mui/icons-material/AddLocationAltTwoTone";
import React from "react";

import SearchForLocation, {
  LocationSearchData,
} from "./components/SearchForLocation";
import DrawerOnSelectLocation from "./components/DrawerOnSelectLocation";

function TabsNavigation({
  value,
  handleChange,
}: {
  value: number;
  handleChange(event: React.SyntheticEvent, newValue: number): void;
}) {
  return (
    <Grid container justifyContent="center" alignItems="center">
      <Grid item>
        <Tabs
          value={value}
          onChange={handleChange}
          aria-label="icon position tabs example"
        >
          <Tab
            icon={<MyLocationTwoToneIcon />}
            iconPosition="start"
            label="Your locations"
          />
          <Tab
            icon={<AddLocationAltTwoToneIcon />}
            iconPosition="end"
            label="Subscribe to location"
          />
        </Tabs>
      </Grid>
    </Grid>
  );
}

type SubscribeToLocationProps = {
  navigateBackToSubscribedLocations: () => void;
};
export default function SubscribeToLocation({
  navigateBackToSubscribedLocations,
}: SubscribeToLocationProps) {
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

  const onSuccessfulSubscription = () => {
    mutate("/locations/list/subscribed");
    navigateBackToSubscribedLocations();
  };
  return (
    <div>
      <SearchForLocation onSelectLocation={onSelectLocation} />
      {locationSelected && (
        <DrawerOnSelectLocation
          drawerState={openConfirmation}
          setDrawerState={setDrawerState}
          location={locationSelected}
          onSuccessfulSubscription={onSuccessfulSubscription}
        />
      )}
    </div>
  );
}
