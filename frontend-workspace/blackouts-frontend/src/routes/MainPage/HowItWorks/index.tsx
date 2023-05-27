import * as React from "react";
import Dialog from "@mui/material/Dialog";
import AppBar from "@mui/material/AppBar";
import Toolbar from "@mui/material/Toolbar";
import IconButton from "@mui/material/IconButton";
import Typography from "@mui/material/Typography";
import CloseIcon from "@mui/icons-material/Close";
import Slide from "@mui/material/Slide";
import { TransitionProps } from "@mui/material/transitions";
import { blue } from "@mui/material/colors";
import Button from "@mui/material/Button";
import Box from "@mui/material/Box";

import Timeline from "@mui/lab/Timeline";
import TimelineItem from "@mui/lab/TimelineItem";
import TimelineSeparator from "@mui/lab/TimelineSeparator";
import TimelineConnector from "@mui/lab/TimelineConnector";
import TimelineContent from "@mui/lab/TimelineContent";
import TimelineDot from "@mui/lab/TimelineDot";

import HistoryTwoToneIcon from "@mui/icons-material/HistoryTwoTone";
import AddLocationAltTwoToneIcon from "@mui/icons-material/AddLocationAltTwoTone";
import MarkEmailUnreadTwoToneIcon from "@mui/icons-material/MarkEmailUnreadTwoTone";

export default function HowItWorksCallout() {
  return (
    <div>
      <Box
        sx={{
          maxWidth: "700px",
          margin: "auto",
          backgroundColor: blue[50],
          padding: "20px",
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          borderRadius: "10px",
          marginBottom: "10px",
        }}
      >
        <Typography variant={"subtitle1"}>
          Click here to see how it works
        </Typography>
        <ExplanationDialog />
      </Box>
    </div>
  );
}

const Transition = React.forwardRef(function Transition(
  props: TransitionProps & {
    children: React.ReactElement;
  },
  ref: React.Ref<unknown>
) {
  return <Slide direction="up" ref={ref} {...props} />;
});

function ExplanationDialog() {
  const [open, setOpen] = React.useState(false);

  const handleClickOpen = () => {
    setOpen(true);
  };

  const handleClose = () => {
    setOpen(false);
  };

  return (
    <div>
      <Button variant={"outlined"} onClick={handleClickOpen}>
        How it works
      </Button>

      <Dialog
        fullScreen
        open={open}
        onClose={handleClose}
        TransitionComponent={Transition}
      >
        <AppBar position={"sticky"}>
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
              How it works
            </Typography>
            <Button autoFocus color="inherit" onClick={handleClose}>
              Close
            </Button>
          </Toolbar>
        </AppBar>
        <div>
          <Timeline>
            <TimelineItem>
              <TimelineSeparator>
                <TimelineConnector />
                <TimelineDot>
                  <AddLocationAltTwoToneIcon />
                </TimelineDot>
                <TimelineConnector />
              </TimelineSeparator>

              <TimelineContent sx={{ py: "12px", px: 2 }}>
                <Typography variant="h6" component="span">
                  Subscribe to a location
                </Typography>
                <Typography>
                  To subscribe, click the "Subscribe to location" tab, search
                  and select your desired location. Feel free to subscribe to
                  multiple locations.
                </Typography>
              </TimelineContent>
            </TimelineItem>
            <TimelineItem position={"left"}>
              <TimelineSeparator>
                <TimelineConnector />
                <TimelineDot>
                  <HistoryTwoToneIcon />
                </TimelineDot>
                <TimelineConnector />
              </TimelineSeparator>
              <TimelineContent sx={{ py: "12px", px: 2 }}>
                <Typography variant="h6" component="span">
                  Monitoring of the KPLC website
                </Typography>
                <Typography>
                  We continuously monitor the KPLC website to detect any newly
                  scheduled power outages automatically.
                </Typography>
              </TimelineContent>
            </TimelineItem>
            <TimelineItem>
              <TimelineSeparator>
                <TimelineConnector />
                <TimelineDot>
                  <MarkEmailUnreadTwoToneIcon />
                </TimelineDot>
                <TimelineConnector />
              </TimelineSeparator>
              <TimelineContent sx={{ py: "12px", px: 2 }}>
                <Typography variant="h6" component="span">
                  Receiving Notifications
                </Typography>
                <Typography>
                  You'll be notified when your subscribed location(s) or
                  surrounding areas are affected. We currently use email
                  notifications, but we'll be expanding the list of notification
                  channels soon.
                </Typography>
              </TimelineContent>
            </TimelineItem>
          </Timeline>
        </div>
      </Dialog>
    </div>
  );
}
