import axios from "axios";
import useSWRMutation from "swr/mutation";
import { useToken } from "../../../providers/Auth";
import { instance } from "../../../axios";

type Props = {
  token: string;
};

async function subscribeToLocation(url: string, { arg }: { arg: Props }) {
  return instance.delete(url, {
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${arg.token}`,
    },
  });
}

export function useDeleteLocationSubscription(locationId: string) {
  const token = useToken();
  const { trigger, isMutating } = useSWRMutation(
    `/locations/primary_location/${locationId}`,
    subscribeToLocation
  );

  const handleDeleteSubscribedLocation = async () => {
    return await trigger({ token });
  };

  return {
    isMutating,
    trigger: handleDeleteSubscribedLocation,
  };
}
