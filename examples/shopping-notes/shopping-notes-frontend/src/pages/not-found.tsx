import { Link } from "react-router-dom";
import type { CSSProperties } from "react";

const containerStyle: CSSProperties = {
  backgroundColor: "#fafafa",
  display: "flex",
  flexDirection: "column",
  gap: "16px",
  alignItems: "center",
  justifyContent: "center",
  minHeight: "100vh",
};

const codeStyle: CSSProperties = {
  fontWeight: 700,
  color: "#1c1c1c",
  fontSize: "64px",
  margin: 0,
};

const titleStyle: CSSProperties = {
  fontWeight: 500,
  color: "#1c1c1c",
  fontSize: "20px",
  margin: 0,
};

const descriptionStyle: CSSProperties = {
  fontWeight: 400,
  color: "#666",
  fontSize: "14px",
  textAlign: "center",
  width: "280px",
  margin: 0,
};

const linkStyle: CSSProperties = {
  backgroundColor: "#296bfa",
  color: "#fff",
  fontWeight: 500,
  fontSize: "14px",
  padding: "10px 20px",
  borderRadius: "8px",
  textDecoration: "none",
};

export default function NotFound() {
  return (
    <div style={containerStyle}>
      <p style={codeStyle}>404</p>
      <p style={titleStyle}>Page not found</p>
      <p style={descriptionStyle}>
        The page you're looking for doesn't exist or has moved.
      </p>
      {/* @spec impl fe-not-found-page#not-found-page-links-to-main-page */}
      <Link to="/" style={linkStyle}>
        Back to main page
      </Link>
    </div>
  );
}
