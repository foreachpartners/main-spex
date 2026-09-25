import { createApp } from "./app.js";

const port = 3000;

createApp().listen({ port }, () => {
  console.log(`Serves http://localhost:${port}`);
});
