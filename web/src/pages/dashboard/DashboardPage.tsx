import { Card, Container, Grid, Text } from "@mantine/core";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useRemoteContext } from "../../remote-context";

interface AlbumsByYearChartItem {
  year: number;
  count: number;
}

const primaryChartColor = "#5B5BD6";
const secondaryChartColor = "#C2298A";

const AlbumsByYearChart = ({ data }: { data: AlbumsByYearChartItem[] }) => {
  data.sort((a, b) => a.year - b.year);
  return (
    <Card padding="sm" shadow="xs" withBorder>
      <Text mb="md">Albums by Release Year</Text>
      <ResponsiveContainer width="100%" height={350}>
        <BarChart
          data={data}
          margin={{
            left: -10,
            right: 10,
          }}
        >
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="year" type="number" domain={["dataMin", "dataMax"]} />
          <YAxis type="number" />
          <Tooltip />
          <Bar
            dataKey="count"
            fill={primaryChartColor}
            stroke={primaryChartColor}
          />
        </BarChart>
      </ResponsiveContainer>
    </Card>
  );
};

const CountCard = ({ title, count }: { title: string; count: number }) => {
  const formattedCount = count.toLocaleString();

  return (
    <Card padding="sm" shadow="xs" withBorder radius="lg">
      <Text size="xl" weight="bold">
        {formattedCount}
      </Text>
      <Text size="sm">{title}</Text>
    </Card>
  );
};

export const DashboardPage = () => {
  const { albumMonitor } = useRemoteContext();

  return (
    <Container size="xl" py="lg">
      <Grid gutter="xl">
        <Grid.Col span={4} lg={2}>
          <CountCard title="Albums" count={albumMonitor.getAlbumCount()} />
        </Grid.Col>

        <Grid.Col span={4} lg={2}>
          <CountCard title="Artists" count={albumMonitor.getArtistCount()} />
        </Grid.Col>

        <Grid.Col span={4} lg={2}>
          <CountCard title="Genres" count={albumMonitor.getGenreCount()} />
        </Grid.Col>

        <Grid.Col span={4} lg={2}>
          <CountCard
            title="Descriptors"
            count={albumMonitor.getDescriptorCount()}
          />
        </Grid.Col>
        <Grid.Col span={4} lg={2}>
          <CountCard
            title="Languages"
            count={albumMonitor.getLanguageCount()}
          />
        </Grid.Col>
        <Grid.Col span={4} lg={2}>
          <CountCard
            title="Duplicate Albums"
            count={albumMonitor.getDuplicateCount()}
          />
        </Grid.Col>
        <Grid.Col span={12} lg={6}>
          <AlbumsByYearChart
            data={albumMonitor.getAggregatedYearsList().map((item) => ({
              year: Number(item.getName()),
              count: item.getCount(),
            }))}
          />
        </Grid.Col>
        <Grid.Col span={12} lg={6}>
          <Card padding="sm" shadow="xs" withBorder>
            <Text mb="md">Albums by Genre</Text>
            <ResponsiveContainer width="100%" height={350}>
              <BarChart
                data={albumMonitor
                  .getAggregatedGenresList()
                  .map((item) => item.toObject())
                  .slice(0, 10)}
                layout="vertical"
                margin={{
                  left: -10,
                  right: 10,
                }}
              >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis type="number" />
                <YAxis dataKey="name" type="category" width={175} />
                <Tooltip />
                <Legend />
                <Bar
                  dataKey="primaryGenreCount"
                  fill={primaryChartColor}
                  stackId="a"
                  name="Primary Genre Count"
                />
                <Bar
                  dataKey="secondaryGenreCount"
                  fill={secondaryChartColor}
                  stackId="a"
                  name="Secondary Genre Count"
                />
              </BarChart>
            </ResponsiveContainer>
          </Card>
        </Grid.Col>
        <Grid.Col span={12} lg={4}>
          <Card padding="sm" shadow="xs" withBorder>
            <Text mb="md">Albums by Descriptor</Text>
            <ResponsiveContainer width="100%" height={350}>
              <BarChart
                data={albumMonitor
                  .getAggregatedDescriptorsList()
                  .map((item) => item.toObject())
                  .slice(0, 10)}
                layout="vertical"
                margin={{
                  left: -10,
                  right: 10,
                }}
              >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis type="number" />
                <YAxis dataKey="name" type="category" width={125} />
                <Tooltip />
                <Bar dataKey="count" fill={primaryChartColor} stackId="a" />
              </BarChart>
            </ResponsiveContainer>
          </Card>
        </Grid.Col>
        <Grid.Col span={12} lg={4}>
          <Card padding="sm" shadow="xs" withBorder>
            <Text mb="md">Albums by Language</Text>
            <ResponsiveContainer width="100%" height={350}>
              <BarChart
                data={albumMonitor
                  .getAggregatedLanguagesList()
                  .map((item) => item.toObject())
                  .slice(0, 10)}
                layout="vertical"
                margin={{
                  left: -10,
                  right: 10,
                }}
              >
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis type="number" />
                <YAxis dataKey="name" type="category" width={125} />
                <Tooltip />
                <Bar dataKey="count" fill={primaryChartColor} stackId="a" />
              </BarChart>
            </ResponsiveContainer>
          </Card>
        </Grid.Col>
      </Grid>
    </Container>
  );
};
