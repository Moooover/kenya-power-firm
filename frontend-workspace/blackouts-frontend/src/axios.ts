import axios from "axios";
const baseURL: string = `${import.meta.env.VITE_API_URL}/api`;

export const instance = axios.create({ baseURL });
instance.defaults.headers.common["Content-Type"] = "application/json";
