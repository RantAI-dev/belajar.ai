import { Link } from "react-router";
import type { LinkComponent } from "../ui";

/** ui and features render <AppLink>; in the app it is a client-side router link. */
export const RouterLink: LinkComponent = ({ href, children, ...rest }) => (
  <Link to={href} {...rest}>
    {children}
  </Link>
);
