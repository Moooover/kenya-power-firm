import * as React from "react";
import Button from "@mui/material/Button";
import Dialog from "@mui/material/Dialog";
import DialogActions from "@mui/material/DialogActions";
import DialogContent from "@mui/material/DialogContent";
import DialogContentText from "@mui/material/DialogContentText";
import DialogTitle from "@mui/material/DialogTitle";
import { useDeleteLocationSubscription } from "./useDeleteSubscribedLocation";
import { mutate } from "swr";
import { useState } from "react";

type Props = {
  open: boolean;
  closeDialog: () => void;
  location: { id: string; name: string };
};
export default function UnsubscribeDialog({
  open,
  closeDialog,
  location,
}: Props) {
  const [loading, setLoading] = useState(false);
  const { trigger } = useDeleteLocationSubscription(location.id);
  const handleDeleteSubscribed = async () => {
    setLoading(true);
    await trigger();
    mutate("/locations/list/subscribed");
  };
  return (
    <div>
      <Dialog
        open={open}
        onClose={closeDialog}
        aria-labelledby="alert-dialog-title"
        aria-describedby="alert-dialog-description"
      >
        <DialogTitle id="alert-dialog-title">{`Unsubscribe from ${location.name} alerts?`}</DialogTitle>
        <DialogContent>
          <DialogContentText id="alert-dialog-description">
            You will no longer be notified when {location.name} will have power
            interruptions.
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button disabled={loading} onClick={closeDialog}>
            Cancel
          </Button>
          <Button disabled={loading} onClick={handleDeleteSubscribed} autoFocus>
            {loading ? "Please wait.." : "Unsubscribe"}
          </Button>
        </DialogActions>
      </Dialog>
    </div>
  );
}
