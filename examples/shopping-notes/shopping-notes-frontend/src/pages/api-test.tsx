import { useEffect, useState } from "react";

export default function ApiTest() {
  const [apiResponse, setApiResponse] = useState<string | null>(null);

  useEffect(() => {
    fetch("http://localhost:3000/")
      .then((response) => response.text())
      .then(setApiResponse);
  }, []);

  return <div>{apiResponse}</div>;
}
