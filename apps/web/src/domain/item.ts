export type ItemIcon = {
  sourceFileName: string;
  canonicalFileName: string;
  cdnUrl: string;
  source: "mapping_icon" | "html_source_match" | "missing";
};

export type Item = {
  itemId: number;
  name: string;
  examine: string | null;
  members: boolean;
  buyLimit: number | null;
  lowAlch: number | null;
  highAlch: number | null;
  value: number | null;
  icon: ItemIcon | null;
};
