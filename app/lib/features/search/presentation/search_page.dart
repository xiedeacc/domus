import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:go_router/go_router.dart';

import '../../../models/asset.dart';
import '../../timeline/widgets/asset_thumbnail.dart';
import '../data/search_repository.dart';

/// Search: metadata filters (filename, city, camera, date) backed by
/// POST /search/metadata. Smart (CLIP) search is a server feature Domus
/// does not provide — the tab is hidden when /server/features reports it off.
class SearchPage extends ConsumerStatefulWidget {
  const SearchPage({super.key});

  @override
  ConsumerState<SearchPage> createState() => _SearchPageState();
}

class _SearchPageState extends ConsumerState<SearchPage> {
  final _controller = TextEditingController();
  Future<List<Asset>>? _future;

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: TextField(
          controller: _controller,
          decoration: const InputDecoration(
            hintText: 'Search your photos',
            border: InputBorder.none,
          ),
          onSubmitted: (query) {
            setState(() {
              _future = ref.read(searchRepositoryProvider).search(query);
            });
          },
        ),
        actions: [
          IconButton(
            icon: const Icon(Icons.map_outlined),
            onPressed: () => context.push('/map'),
          ),
          IconButton(
            icon: const Icon(Icons.folder_outlined),
            onPressed: () => context.push('/folders'),
          ),
        ],
      ),
      body: _future == null
          ? const Center(child: Text('Search by filename, place or date'))
          : FutureBuilder(
              future: _future,
              builder: (context, snapshot) {
                if (snapshot.connectionState != ConnectionState.done) {
                  return const Center(child: CircularProgressIndicator());
                }
                if (snapshot.hasError) {
                  return Center(child: Text('${snapshot.error}'));
                }
                final assets = snapshot.data ?? const [];
                if (assets.isEmpty) {
                  return const Center(child: Text('No results'));
                }
                return GridView.builder(
                  gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
                    maxCrossAxisExtent: 160,
                    mainAxisSpacing: 2,
                    crossAxisSpacing: 2,
                  ),
                  itemCount: assets.length,
                  itemBuilder: (context, i) => AssetThumbnail(
                    asset: assets[i],
                    onTap: () => context.push('/asset/${assets[i].id}'),
                  ),
                );
              },
            ),
    );
  }
}
