import useSWRMutation from "swr/mutation";
import { useToken } from "../../providers/Auth";
import { AxiosError } from "axios";
import React from "react";
import useSWR from "swr";
import { instance } from "../../axios";

type SubscriptionStatus = "Pending" | "Success" | "Failure" | "NotFound";

type StatusWrapper = {
  data: SubscriptionStatus;
};

type RequestData = {
  location: string;
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
  return instance
    .post<SubscribeResponse>(
      url,
      { location: data.location },
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
    "/locations/subscribe",
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
    return trigger({ data, token }).then((data) => {
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
