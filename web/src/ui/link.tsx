import { type AnchorHTMLAttributes, createContext, type ReactNode, useContext } from "react";

export type LinkProps = Omit<AnchorHTMLAttributes<HTMLAnchorElement>, "href"> & {
  readonly href: string;
};
export type LinkComponent = (props: LinkProps) => ReactNode;

const PlainLink: LinkComponent = ({ href, children, ...rest }) => (
  <a href={href} {...rest}>
    {children}
  </a>
);

const LinkContext = createContext<LinkComponent>(PlainLink);

/** The app supplies its router's link here, so ui and features never import the router. */
export function LinkProvider({
  component,
  children,
}: {
  readonly component: LinkComponent;
  readonly children: ReactNode;
}) {
  return <LinkContext.Provider value={component}>{children}</LinkContext.Provider>;
}

export function AppLink(props: LinkProps) {
  const Component = useContext(LinkContext);
  return <Component {...props} />;
}
