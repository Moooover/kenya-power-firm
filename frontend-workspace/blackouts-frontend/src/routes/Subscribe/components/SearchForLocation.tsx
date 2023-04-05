import SearchBox from "../../../components/SearchBox";
import useSWR from "swr";
import React from "react";
import { useDebounce } from "../../../hooks/useDebounce";
import { AxiosError } from "axios";
import ListOfLocations from "./ListOfLocations";
import { Typography } from "@mui/material";

export type LocationSearchData = {
  id: string;
  name: string;
  address: string;
};

type Status = "Pending" | "Success" | "Failure" | "NotFound";
export type SearchResponse = {
  items: LocationSearchData[];
  status: Status;
};

function useAutoComplete() {
  const [searchTerm, setSearchTerm] = React.useState("");
  const debouncedSearchTerm = useDebounce(searchTerm, 500);
  const { data, error, mutate } = useSWR<SearchResponse, AxiosError>(
    debouncedSearchTerm.trim()
      ? `/locations/search?term=${debouncedSearchTerm}`
      : null,
    null,
    {
      onSuccess(data, key, config) {
        if (data.status === "Pending") {
          mutate();
        }
      },
    }
  );

  return {
    data,
    error,
    searchTerm,
    setSearchTerm,
  };
}

type Props = {
  onSelectLocation: (location: LocationSearchData) => void;
};

export default function SearchForLocation(props: Props) {
  const { data, searchTerm, setSearchTerm } = useAutoComplete();
  const handleChangeSearchTerm = (val: string) => {
    setSearchTerm(val);
  };

  const onSelectLocation = (location: LocationSearchData) => {
    props.onSelectLocation(location);
    setSearchTerm("");
  };

  return (
    <div>
      <SearchBox
        handleChangeSearchTerm={handleChangeSearchTerm}
        value={searchTerm}
      />

      {data && data.status === "Success" && data.items.length > 0 && (
        <ListOfLocations items={data.items} onClick={onSelectLocation} />
      )}
      {data && data.status === "Pending" && (
        <div>
          <Typography>Loading..</Typography>
        </div>
      )}
      {data && data.status === "Failure" && (
        <div>
          <Typography>Something went wrong, please try again later</Typography>
        </div>
      )}
      {data && data.status === "NotFound" && (
        <div>
          <Typography>The location was not found</Typography>
        </div>
      )}
    </div>
  );
}
