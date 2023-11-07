import {
  Card,
  Flex,
  Pagination,
  Select,
  Switch,
  Text,
  TextInput,
} from "@mantine/core";
import { useEffect, useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Profile } from "../../../proto/lute_pb";
import { ProfileAlbumsListItem } from "./ProfileAlbumsListItem";
import { ProfileDetailsCard } from "./ProfileDetailsCard";
import { ProfileAlbumsList } from "./types";
import { useDebounce } from "./use-debounce";

interface ProfileAlbumsProps {
  profile: Profile;
  list: ProfileAlbumsList;
}

const getUpdatedQueryString = (updates: Record<string, any>) => {
  const url = new URL(window.location.href);
  const searchParams = new URLSearchParams(url.search);
  for (const [key, value] of Object.entries(updates)) {
    if (value !== undefined) {
      searchParams.set(key, value);
    }
  }
  return "?" + searchParams.toString();
};

const AlbumSearchInput = ({ value }: { value: string }) => {
  const [searchValue, setSearchValue] = useState(value);
  const debouncedSearchValue = useDebounce(searchValue, 250);
  const navigate = useNavigate();
  useEffect(() => {
    navigate(
      getUpdatedQueryString({
        search: debouncedSearchValue,
        page: value !== debouncedSearchValue ? 1 : undefined,
      }),
    );
  }, [debouncedSearchValue]);
  useEffect(() => {
    if (!value) {
      setSearchValue("");
    }
  }, [value]);

  return (
    <TextInput
      placeholder="Search"
      variant="filled"
      value={searchValue}
      onChange={(e) => {
        setSearchValue(e.currentTarget.value);
      }}
    />
  );
};

const PageSizeSelect = ({ list }: { list: ProfileAlbumsList }) => {
  const navigate = useNavigate();
  return (
    <Text>
      Showing{" "}
      <Select
        data={[
          { value: "5", label: "5" },
          { value: "10", label: "10" },
          { value: "25", label: "25" },
          { value: "50", label: "50" },
        ]}
        defaultValue={list.pageSize.toString()}
        styles={{
          root: {
            width: 70,
            display: "inline-block",
          },
          rightSection: {
            paddingLeft: 4,
          },
        }}
        onChange={(pageSize) => {
          navigate(
            getUpdatedQueryString({
              pageSize,
              page: 1,
            }),
          );
        }}
      />
      of {list.total} albums
    </Text>
  );
};

const getControlHref = (control: string, list: ProfileAlbumsProps["list"]) => {
  switch (control) {
    case "first":
      return getUpdatedQueryString({ page: 1 });
    case "last":
      return getUpdatedQueryString({ page: list.pageCount });
    case "next":
      return getUpdatedQueryString({ page: list.page + 1 });
    case "previous":
      return getUpdatedQueryString({ page: list.page - 1 });
    default:
      return undefined;
  }
};

const SearchModeSwitch = ({ list }: { list: ProfileAlbumsList }) => {
  const navigate = useNavigate();
  return (
    <Switch
      onLabel={<Text>New</Text>}
      offLabel={<Text>Existing</Text>}
      size="lg"
      radius="md"
      styles={{
        track: {
          background: "#DBDBDB",
          fontWeight: "normal",
          fontSize: 14,
          width: 90,
          textAlign: "center",
          minHeight: "2.25rem",
        },
      }}
      checked={list.searchMode === "new"}
      onChange={(e) => {
        navigate(
          getUpdatedQueryString({
            searchMode: e.currentTarget.checked ? "new" : "existing",
            page: 1,
          }),
        );
      }}
    />
  );
};

export const ProfileAlbums = ({ profile, list }: ProfileAlbumsProps) => {
  return (
    <ProfileDetailsCard
      label="Albums"
      footer={
        <Flex justify="space-between" align="center">
          <Pagination
            value={list.page}
            total={list.pageCount}
            getItemProps={(page) => ({
              component: Link,
              to: getUpdatedQueryString({ page }),
            })}
            getControlProps={(control) => {
              const to = getControlHref(control, list);
              return to ? { component: Link, to } : {};
            }}
          />
          <PageSizeSelect list={list} />
        </Flex>
      }
    >
      <Card.Section withBorder inheritPadding py="xs">
        <Flex gap="md" align="center">
          <div
            style={{
              flex: 1,
            }}
          >
            <AlbumSearchInput value={list.search} />
          </div>
          <SearchModeSwitch list={list} />
        </Flex>
      </Card.Section>
      <div>
        {list.albums.map((album, i) => (
          <ProfileAlbumsListItem
            key={album.getFileName()}
            album={album}
            profile={profile}
            searchMode={list.searchMode}
            factor={profile.getAlbumsMap().get(album.getFileName()) || 0}
            hasBorder={i !== list.albums.length - 1}
          />
        ))}
      </div>
    </ProfileDetailsCard>
  );
};
