import { Link } from "react-router-dom";

export default function NotFound() {
  return (
    <div className="not-found">
      <p className="not-found__code">404</p>
      <p className="not-found__title">Page not found</p>
      <p className="not-found__description">
        The page you're looking for doesn't exist or has moved.
      </p>
      {/* @spec impl fe-not-found-page#not-found-page-links-to-main-page */}
      <Link to="/" className="not-found__link">
        Back to main page
      </Link>
    </div>
  );
}
