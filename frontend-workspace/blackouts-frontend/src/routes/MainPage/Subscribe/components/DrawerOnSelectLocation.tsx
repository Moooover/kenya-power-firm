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
import { LocationSearchData } from "./SearchForLocation";
import styled from "@emotion/styled";
import { useSubscribeToLocation } from "../useSubscribe";
import { mutate } from "swr";
import { useNavigate } from "react-router-dom";
import DialogActions from "@mui/material/DialogActions";
import DialogTitle from "@mui/material/DialogTitle";
import DialogContent from "@mui/material/DialogContent";
import DialogContentText from "@mui/material/DialogContentText";

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
  onSuccessfulSubscription: () => void;
};

export default function DrawerOnSelectLocation(props: Props) {
  const { location, setDrawerState, onSuccessfulSubscription } = props;
  const navigate = useNavigate();

  const { isLoading, trigger } = useSubscribeToLocation({
    onSuccess: onSuccessfulSubscription,
  });
  const handleClose = () => {
    if (isLoading) return;
    setDrawerState(false);
  };

  const handleSubscribeToLocation = async () => {
    try {
      await trigger({
        location: location.id,
      });
    } catch (e) {
      alert("Something went wrong, please try again later");
    }
  };

  return (
    <div>
      <Dialog
        open={props.drawerState}
        onClose={handleClose}
        TransitionComponent={Transition}
      >
        <DialogTitle>Subscribe to {location.name} alerts ?</DialogTitle>
        <DialogContent>
          <DialogContentText id="alert-dialog-description">
            You will be notified when <b>{location.name}</b> or nearby locations
            are going to have power interruptions in advance.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button disabled={isLoading} onClick={handleClose}>
            Cancel
          </Button>

          <Button
            disabled={isLoading}
            autoFocus
            onClick={handleSubscribeToLocation}
          >
            {isLoading ? "Loading..." : "Subscribe"}
          </Button>
        </DialogActions>
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
