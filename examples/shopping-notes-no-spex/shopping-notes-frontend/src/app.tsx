import { Route, Routes } from "react-router-dom";
import Home from "./pages/home";
import ApiTest from "./pages/api-test";

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Home />} />
      <Route path="/api-test" element={<ApiTest />} />
    </Routes>
  );
}
