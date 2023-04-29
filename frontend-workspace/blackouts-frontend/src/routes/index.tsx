import React from "react";
import { createBrowserRouter } from "react-router-dom";
import Layout from "../components/Layout";
import MainPage from "./MainPage";

export const router = createBrowserRouter([
  {
    path: "/",
    element: (
      <Layout>
        <MainPage />
      </Layout>
    ),
  },
]);
