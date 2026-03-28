"use client";

import { BarChart, Bar, ResponsiveContainer } from "recharts";

interface SparklineBarProps {
  data: number[];
  color?: string;
  height?: number;
}

export default function SparklineBar({
  data,
  color = "#76d6d5",
  height = 40,
}: SparklineBarProps) {
  const chartData = data.map((value, index) => ({ index, value }));

  return (
    <ResponsiveContainer width="100%" height={height}>
      <BarChart data={chartData}>
        <Bar dataKey="value" fill={color} radius={[1, 1, 0, 0]} />
      </BarChart>
    </ResponsiveContainer>
  );
}
