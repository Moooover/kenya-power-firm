import { mutate } from "swr";
import React from "react";

import SearchForLocation, {
  LocationSearchData,
} from "./components/SearchForLocation";
import DrawerOnSelectLocation from "./components/DrawerOnSelectLocation";

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
