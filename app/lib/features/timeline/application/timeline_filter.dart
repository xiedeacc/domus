import 'package:flutter/material.dart';
import 'package:intl/intl.dart';

import '../../../models/asset.dart';

enum TimelineGranularity { year, month, day }

extension TimelineGranularityLabel on TimelineGranularity {
  String get label => switch (this) {
    TimelineGranularity.year => 'Year',
    TimelineGranularity.month => 'Month',
    TimelineGranularity.day => 'Day',
  };
}

class TimelineDateRange {
  const TimelineDateRange({required this.start, required this.end});

  final DateTime start;
  final DateTime end;

  bool includes(DateTime? value) {
    if (value == null) return false;
    final normalized = DateTime(value.year, value.month, value.day);
    return !normalized.isBefore(start) && !normalized.isAfter(end);
  }

  bool includesAsset(Asset asset) => includes(asset.localDateTime);

  bool overlapsMonthBucket(String bucket) {
    final bucketStart = DateTime.tryParse(bucket);
    if (bucketStart == null) return true;
    final bucketEnd = DateTime(bucketStart.year, bucketStart.month + 1, 0);
    return !bucketEnd.isBefore(start) && !bucketStart.isAfter(end);
  }

  String label(TimelineGranularity granularity) {
    final formatter = switch (granularity) {
      TimelineGranularity.year => DateFormat.y(),
      TimelineGranularity.month => DateFormat.yMMM(),
      TimelineGranularity.day => DateFormat.yMMMd(),
    };
    final startText = formatter.format(start);
    final endText = formatter.format(end);
    return startText == endText ? startText : '$startText - $endText';
  }
}

DateTime timelineSectionStart(DateTime value, TimelineGranularity granularity) {
  return switch (granularity) {
    TimelineGranularity.year => DateTime(value.year),
    TimelineGranularity.month => DateTime(value.year, value.month),
    TimelineGranularity.day => DateTime(value.year, value.month, value.day),
  };
}

TimelineDateRange timelineRangeFor(
  DateTime value,
  TimelineGranularity granularity,
) {
  final start = timelineSectionStart(value, granularity);
  return switch (granularity) {
    TimelineGranularity.year => TimelineDateRange(
      start: start,
      end: DateTime(start.year, 12, 31),
    ),
    TimelineGranularity.month => TimelineDateRange(
      start: start,
      end: DateTime(start.year, start.month + 1, 0),
    ),
    TimelineGranularity.day => TimelineDateRange(start: start, end: start),
  };
}

String timelineSectionTitle(DateTime value, TimelineGranularity granularity) {
  return switch (granularity) {
    TimelineGranularity.year => DateFormat.y().format(value),
    TimelineGranularity.month => DateFormat.yMMMM().format(value),
    TimelineGranularity.day => DateFormat.yMMMMEEEEd().format(value),
  };
}

TimelineDateRange normalizeDateRange(
  DateTimeRange range,
  TimelineGranularity granularity,
) {
  final start = DateTime(range.start.year, range.start.month, range.start.day);
  final end = DateTime(range.end.year, range.end.month, range.end.day);
  return switch (granularity) {
    TimelineGranularity.year => TimelineDateRange(
      start: DateTime(start.year),
      end: DateTime(end.year, 12, 31),
    ),
    TimelineGranularity.month => TimelineDateRange(
      start: DateTime(start.year, start.month),
      end: DateTime(end.year, end.month + 1, 0),
    ),
    TimelineGranularity.day => TimelineDateRange(start: start, end: end),
  };
}
