from datetime import datetime, timedelta, timezone
import unittest
from unittest.mock import patch
import xml.etree.ElementTree as ET

import gen_star_history as charts


class StarHistoryTests(unittest.TestCase):
    def test_weekly_api_pages_are_all_collected(self):
        first = [{"week": index, "days": [0] * 7} for index in range(30)]
        second = [{"week": 31, "days": [1, 0, 0, 0, 0, 0, 0]}]
        with patch.object(charts, "request_json", side_effect=[first, second]) as request:
            self.assertEqual(charts.fetch_history("owner/repo"), first + second)
        self.assertEqual(request.call_count, 2)
        self.assertTrue(request.call_args_list[1].args[0].endswith("page=2"))

    def test_reversed_weeks_become_chronological_cumulative_stars(self):
        weeks = [
            {"week": 7 * 86400, "days": [3, 0, 0, 0, 0, 0, 0]},
            {"week": 0, "days": [1, 2, 0, 0, 0, 0, 0]},
        ]
        series = charts.cumulative_series(weeks)
        self.assertEqual([count for _, count in series], [1, 3, 6])
        self.assertEqual(series, sorted(series))

    def test_sampling_preserves_history_endpoints(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        series = [(start + timedelta(days=index), index + 1) for index in range(500)]
        sampled = charts.sample_series(series)
        self.assertEqual(len(sampled), 160)
        self.assertEqual(sampled[0], series[0])
        self.assertEqual(sampled[-1], series[-1])

    def test_empty_and_single_day_charts_are_valid_and_deterministic(self):
        start = datetime(2025, 1, 1, tzinfo=timezone.utc)
        for theme in charts.COLORS:
            for series in [[], [(start, 3)]]:
                with self.subTest(theme=theme, stars=series):
                    svg = charts.render_svg("owner/repo & chart", series, start, theme)
                    root = ET.fromstring(svg)
                    self.assertEqual(root.attrib["viewBox"], "0 0 820 420")
                    title = root.find("{http://www.w3.org/2000/svg}title")
                    self.assertIn("owner/repo & chart", title.text)
                    self.assertEqual(
                        svg, charts.render_svg("owner/repo & chart", series, start, theme)
                    )
                    self.assertNotIn("nan", svg)
                    self.assertNotIn("inf", svg)


if __name__ == "__main__":
    unittest.main()
