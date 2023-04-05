import axios from "axios";
import useSWRMutation from "swr/mutation";
import { useToken } from "../../providers/Auth";

type Props = {
  token: string;
};

async function subscribeToLocation(url: string, { arg }: { arg: Props }) {
  return axios.delete(url, {
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${arg.token}`,
    },
  });
}

export function useDeleteLocationSubscription(locationId: string) {
  const token = useToken();
  const { trigger, isMutating } = useSWRMutation(
    `/api/locations/primary_location/${locationId}`,
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
