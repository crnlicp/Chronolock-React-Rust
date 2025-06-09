import { NavLink } from 'react-router';

export const NotFound = () => {
  return (
    <div className="container page_container">
      <div className="notfound">
        <div>
          <h1>404</h1>
        </div>
        <h3>Oops! Page not found</h3>
        <p className="notfound_text">
          The page you are looking for might have been removed.
        </p>
        <NavLink to="/" className="metaportal_fn_button">
          Back to homepage
        </NavLink>
      </div>
    </div>
  );
};
