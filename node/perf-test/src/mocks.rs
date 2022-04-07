use nimbus_primitives::AuthorFilterAPI;
use nimbus_primitives::CompatibleDigestItem;
use nimbus_primitives::NimbusApi;
use nimbus_primitives::NimbusId;
use sc_consensus::block_import::BlockImportParams;
use sc_consensus_manual_seal::ConsensusDataProvider;
use sc_service::Arc;
use sp_api::ProvideRuntimeApi;
use sp_api::TransactionFor;
use sp_inherents::InherentData;
use sp_runtime::traits::Block;
use sp_runtime::Digest;
use sp_runtime::DigestItem;

pub struct MockConsensusDataProvider<C> {
	pub author_id: NimbusId,
	pub client: Arc<C>,
}
impl<B, C> ConsensusDataProvider<B> for MockConsensusDataProvider<C>
where
	B: Block,
	C: ProvideRuntimeApi<B> + Send + Sync,
	C::Api: NimbusApi<B>,
	C::Api: AuthorFilterAPI<B, NimbusId>,
{
	type Transaction = TransactionFor<C, B>;

	fn create_digest(
		&self,
		_parent: &B::Header,
		_inherents: &InherentData,
	) -> Result<Digest, sc_consensus_manual_seal::Error> {
		Ok(Digest {
			logs: vec![DigestItem::nimbus_pre_digest(self.author_id.clone())],
		})
	}

	/// set up the neccessary import params.
	fn append_block_import(
		&self,
		_parent: &B::Header,
		_params: &mut BlockImportParams<B, Self::Transaction>,
		_inherents: &InherentData,
	) -> Result<(), sc_consensus_manual_seal::Error> {
		Ok(())
	}
}
