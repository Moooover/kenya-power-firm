import React from "react";
import { createRoot } from "react-dom/client";
import {
  createBrowserRouter,
  RouterProvider,
  Route,
  Link,
} from "react-router-dom";
import SubscribeToLocation from "./Subscribe/SubscribeToLocation";
import SubscribedLocations from "./SubscribedLocations";
import Layout from "../components/Layout";

export const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <Layout>
        <SubscribeToLocation />
      </Layout>
    ),
  },
  {
    path: "/subscribed",
    element: (
      <Layout>
        <SubscribedLocations />
      </Layout>
    ),
  },
]);
