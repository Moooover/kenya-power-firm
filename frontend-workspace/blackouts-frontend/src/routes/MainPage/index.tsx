import Tabs from "@mui/material/Tabs";
import Tab from "@mui/material/Tab";
import Grid from "@mui/material/Grid";
import Box from "@mui/material/Box";

import MyLocationTwoToneIcon from "@mui/icons-material/MyLocationTwoTone";
import AddLocationAltTwoToneIcon from "@mui/icons-material/AddLocationAltTwoTone";
import React from "react";

import SubscribeToLocation from "./Subscribe";
import SubscribedLocations from "./SubscribedLocations";

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
            iconPosition="end"
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

export default function MainPage() {
  const [value, setValue] = React.useState(0);

  const handleChange = (event: React.SyntheticEvent, newValue: number) => {
    setValue(newValue);
  };

  const navigateBackToSubscribedLocations = () => {
    setValue(0);
  };
  return (
    <Box
      sx={{
        margin: "auto",
        maxWidth: 700,
      }}
    >
      <TabsNavigation value={value} handleChange={handleChange} />
      <Box
        sx={{
          marginTop: "20px",
        }}
      >
        {value === 0 && <SubscribedLocations />}
        {value === 1 && (
          <SubscribeToLocation
            navigateBackToSubscribedLocations={
              navigateBackToSubscribedLocations
            }
          />
        )}
      </Box>
    </Box>
  );
}
