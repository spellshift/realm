import CredentialTable, { CredentialTableProps } from "./CredentialTable";

const CredentialTableGrouped = (props: CredentialTableProps) => (
    <CredentialTable {...props} showLastModified />
);

export default CredentialTableGrouped;
