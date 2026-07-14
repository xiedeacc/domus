import 'package:domus_app/features/timeline/application/timeline_filter.dart';
import 'package:domus_app/models/asset.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('normalizes date range by year, month and day', () {
    final raw = DateTimeRange(
      start: DateTime(2024, 5, 18),
      end: DateTime(2025, 2, 3),
    );

    expect(
      normalizeDateRange(raw, TimelineGranularity.year).start,
      DateTime(2024),
    );
    expect(
      normalizeDateRange(raw, TimelineGranularity.year).end,
      DateTime(2025, 12, 31),
    );
    expect(
      normalizeDateRange(raw, TimelineGranularity.month).start,
      DateTime(2024, 5),
    );
    expect(
      normalizeDateRange(raw, TimelineGranularity.month).end,
      DateTime(2025, 2, 28),
    );
    expect(
      normalizeDateRange(raw, TimelineGranularity.day).start,
      DateTime(2024, 5, 18),
    );
    expect(
      normalizeDateRange(raw, TimelineGranularity.day).end,
      DateTime(2025, 2, 3),
    );
  });

  test('filters both photos and videos by inclusive date range', () {
    final range = TimelineDateRange(
      start: DateTime(2024, 6),
      end: DateTime(2024, 6, 30),
    );
    final image = Asset(
      id: 'image-1',
      type: 'IMAGE',
      ownerId: 'user-1',
      originalFileName: 'IMG_0001.jpg',
      localDateTime: DateTime(2024, 6, 10, 12, 30),
    );
    final video = Asset(
      id: 'video-1',
      type: 'VIDEO',
      ownerId: 'user-1',
      originalFileName: 'VID_0001.mov',
      localDateTime: DateTime(2024, 6, 30, 23, 59),
    );
    final outside = Asset(
      id: 'image-2',
      type: 'IMAGE',
      ownerId: 'user-1',
      originalFileName: 'IMG_0002.jpg',
      localDateTime: DateTime(2024, 7),
    );

    expect(range.includesAsset(image), isTrue);
    expect(range.includesAsset(video), isTrue);
    expect(range.includesAsset(outside), isFalse);
  });

  test('selects whole day, month or year from a timeline checkmark', () {
    final date = DateTime(2026, 7, 14, 18, 30);

    expect(
      timelineRangeFor(date, TimelineGranularity.day).start,
      DateTime(2026, 7, 14),
    );
    expect(
      timelineRangeFor(date, TimelineGranularity.day).end,
      DateTime(2026, 7, 14),
    );
    expect(
      timelineRangeFor(date, TimelineGranularity.month).start,
      DateTime(2026, 7),
    );
    expect(
      timelineRangeFor(date, TimelineGranularity.month).end,
      DateTime(2026, 7, 31),
    );
    expect(
      timelineRangeFor(date, TimelineGranularity.year).start,
      DateTime(2026),
    );
    expect(
      timelineRangeFor(date, TimelineGranularity.year).end,
      DateTime(2026, 12, 31),
    );
  });

  test('detects overlapping timeline month buckets', () {
    final range = TimelineDateRange(
      start: DateTime(2024, 6, 15),
      end: DateTime(2024, 7, 2),
    );

    expect(range.overlapsMonthBucket('2024-05-01'), isFalse);
    expect(range.overlapsMonthBucket('2024-06-01'), isTrue);
    expect(range.overlapsMonthBucket('2024-07-01'), isTrue);
    expect(range.overlapsMonthBucket('2024-08-01'), isFalse);
  });
}
