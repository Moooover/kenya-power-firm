import { Input } from "@mui/material";
import SearchBox from "../components/SearchBox";
import useSWR from "swr";
import React from "react";

function useAutoComplete() {
  const [searchTerm, setSearchTerm] = React.useState("Mi Vida");
  const { data, error } = useSWR(`/locations/search?term=${searchTerm}`);

  return {
    data,
    error,
    searchTerm,
    setSearchTerm,
  };
}

export default function SubscribeToLocation() {
  const { data, setSearchTerm } = useAutoComplete();
  console.log(data);
  return (
    <div>
      <SearchBox />
    </div>
  );
}
