import React from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  RouterProvider,
  Route,
  Link,
} from "react-router-dom";
import SubscribeToLocation from "./subscribe_to_location";

export const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <div>
        <SubscribeToLocation />
      </div>
    ),
  },
]);
