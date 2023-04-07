import * as React from "react";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import IconButton from "@mui/material/IconButton";
import Typography from "@mui/material/Typography";
import CloseIcon from "@mui/icons-material/Close";
import Slide from "@mui/material/Slide";
import { TransitionProps } from "@mui/material/transitions";
import { Box, Grid, Paper } from "@mui/material";
import SearchForLocation, { LocationSearchData } from "./SearchForLocation";
import styled from "@emotion/styled";
import AddLocationAltTwoToneIcon from "@mui/icons-material/AddLocationAltTwoTone";
import AdjuscentLocations from "./AdjuscentLocations";
import { useSubscribeToLocation } from "../useSubscribe";
import { mutate } from "swr";
import { useNavigate } from "react-router-dom";

const Transition = React.forwardRef(function Transition(
  props: TransitionProps & {
    children: React.ReactElement;
  },
  ref: React.Ref<unknown>
) {
  return <Slide direction="up" ref={ref} {...props} />;
});

type Props = {
  drawerState: boolean;
  setDrawerState: (value: boolean) => void;
  location: LocationSearchData;
};

export default function DrawerOnSelectLocation(props: Props) {
  const [adjuscentLocations, setAdjuscentLocations] = React.useState<
    LocationSearchData[]
  >([]);
  const { location, setDrawerState } = props;
  const navigate = useNavigate();

  const onSuccess = () => {
    mutate("/locations/list/subscribed");
    navigate("/subscribed");
  };

  const { isLoading, trigger } = useSubscribeToLocation({
    onSuccess,
  });
  const handleClose = () => {
    if (isLoading) return;
    setDrawerState(false);
    setAdjuscentLocations([]);
  };

  const onSelectAdjuscentLocations = (location: LocationSearchData) => {
    setAdjuscentLocations((existingLocations) => {
      return existingLocations.concat(location);
    });
  };

  const handleDelete = (chipToDelete: LocationSearchData) => () => {
    setAdjuscentLocations((chips) =>
      chips.filter((chip) => chip.id !== chipToDelete.id)
    );
  };

  const handleSubscribeToLocation = async () => {
    await trigger({
      location: location.id,
      nearby_locations: adjuscentLocations.map((location) => location.id),
    });
  };

  return (
    <div>
      <Dialog
        fullScreen
        open={props.drawerState}
        onClose={handleClose}
        TransitionComponent={Transition}
      >
        <AppBar sx={{ position: "relative" }}>
          <Toolbar>
            <IconButton
              edge="start"
              color="inherit"
              onClick={handleClose}
              aria-label="close"
            >
              <CloseIcon />
            </IconButton>
            <Typography sx={{ ml: 2, flex: 1 }} variant="h6" component="div">
              Subscription confirmation
            </Typography>
            <Button
              disabled={isLoading}
              autoFocus
              color="inherit"
              endIcon={<AddLocationAltTwoToneIcon />}
              onClick={handleSubscribeToLocation}
            >
              {isLoading ? "Loading..." : "Subscribe"}
            </Button>
          </Toolbar>
        </AppBar>
        <StyledBoxContainer>
          <Paper variant="outlined">
            <Box>
              <Grid
                container
                spacing={2}
                direction="row"
                justifyContent="center"
                alignItems="center"
              >
                <Grid item xs={1}>
                  <IconButton size="large">
                    <AddLocationAltTwoToneIcon />
                  </IconButton>
                </Grid>
                <Grid item xs={11}>
                  <StyledTypography>
                    Are you sure you want to subscribe to <b>{location.name}</b>{" "}
                    - <b>{location.address} ?</b>
                  </StyledTypography>
                </Grid>
              </Grid>
            </Box>
          </Paper>

          <StyledNearbyBoxContainer>
            <Box>
              <Typography variant="h6">Add nearby Locations</Typography>
              <Typography>
                KPLC might or might not mention the above selected location. To
                increase your chances of being notified, select a couple of
                nearby locations, so you get notified when the nearby locations
                are affected.
              </Typography>
            </Box>
            <StyledAdjuscentLocations>
              {adjuscentLocations.length > 0 && (
                <AdjuscentLocations
                  locations={adjuscentLocations}
                  handleDelete={handleDelete}
                />
              )}
              <Box marginTop={5}>
                <SearchForLocation
                  onSelectLocation={onSelectAdjuscentLocations}
                />
              </Box>
            </StyledAdjuscentLocations>
          </StyledNearbyBoxContainer>
        </StyledBoxContainer>
      </Dialog>
    </div>
  );
}

const StyledBoxContainer = styled(Box)({
  marginTop: 40,
  padding: 30,
});

const StyledTypography = styled(Typography)({
  margin: 10,
});

const StyledNearbyBoxContainer = styled(Box)({
  marginTop: 30,
});

const StyledAdjuscentLocations = styled(Box)({
  marginTop: 10,
});
