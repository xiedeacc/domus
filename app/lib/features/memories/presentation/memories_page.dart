import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

/// "On this day" memories from GET /memories.
class MemoriesPage extends ConsumerWidget {
  const MemoriesPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Memories')),
      // TODO: memoriesProvider → horizontal story cards per memory.
      body: const Center(child: Text('Memories will appear here')),
    );
  }
}
