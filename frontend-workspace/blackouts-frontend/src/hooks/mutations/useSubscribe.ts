import useSWRMutation from "swr/mutation";
import { useAuth, useToken } from "../../providers/Auth";
import axios from "axios";
import { AxiosError } from "axios";
import React from "react";
import useSWR from "swr";

type SubscriptionStatus = "Pending" | "Success" | "Failure" | "NotFound";

type StatusWrapper = {
  data: SubscriptionStatus;
};

type RequestData = {
  location: string;
  nearby_locations: string[];
};

type Props = {
  data: RequestData;
  token: string;
};

type SubscribeResponse = {
  task_id: string;
};
async function subscribeToLocation(url: string, { arg }: { arg: Props }) {
  const data = arg.data;
  return axios
    .post<SubscribeResponse>(
      url,
      { location: data.location, nearby_locations: data.nearby_locations },
      {
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${arg.token}`,
        },
      }
    )
    .then((data) => data.data);
}

type SubscribeProps = {
  onSuccess: Function;
};

export function useSubscribeToLocation(props: SubscribeProps) {
  const [isLoading, setIsLoading] = React.useState(false);
  const [taskId, setTaskId] = React.useState<string | null>(null);
  const token = useToken();
  const { trigger } = useSWRMutation(
    "/api/locations/subscribe",
    subscribeToLocation
  );
  const { data: subscriptionStatus, mutate } = useSWR<
    StatusWrapper,
    AxiosError
  >(taskId ? `/locations/subscribe/progress/${taskId}` : null, null, {
    onSuccess(data, key, config) {
      if (data.data === "Pending") {
        return mutate();
      }
      setIsLoading(false);
      props.onSuccess();
    },
  });

  const handleSubscribe = async (data: RequestData) => {
    setIsLoading(true);
    trigger({ data, token }).then((data) => {
      if (data?.task_id) {
        setTaskId(data.task_id);
      }
    });
  };

  return {
    isLoading,
    trigger: handleSubscribe,
    subscriptionStatus,
  };
}
