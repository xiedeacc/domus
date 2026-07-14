import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/memory_repository.dart';

class MemoriesPage extends ConsumerWidget {
  const MemoriesPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    return Scaffold(
      appBar: AppBar(title: const Text('Memories')),
      body: ref
          .watch(memoriesProvider)
          .when(
            loading: () => const Center(child: CircularProgressIndicator()),
            error: (error, _) => Center(child: Text('$error')),
            data: (memories) {
              if (memories.isEmpty) {
                return const Center(child: Text('No memories today'));
              }
              return ListView.separated(
                padding: const EdgeInsets.all(12),
                itemCount: memories.length,
                separatorBuilder: (_, _) => const SizedBox(height: 16),
                itemBuilder: (context, index) {
                  final memory = memories[index];
                  return Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Padding(
                        padding: const EdgeInsets.symmetric(vertical: 8),
                        child: Text(
                          memory.title,
                          style: Theme.of(context).textTheme.titleMedium,
                        ),
                      ),
                      SizedBox(
                        height: 130,
                        child: ListView.separated(
                          scrollDirection: Axis.horizontal,
                          itemCount: memory.assets.length,
                          separatorBuilder: (_, _) => const SizedBox(width: 4),
                          itemBuilder: (context, i) => SizedBox(
                            width: 130,
                            child: AssetThumbnail(
                              asset: memory.assets[i],
                              onTap: () =>
                                  context.push('/asset/${memory.assets[i].id}'),
                            ),
                          ),
                        ),
                      ),
                    ],
                  );
                },
              );
            },
          ),
    );
  }
}
