use teloxide::prelude::ResponseResult;

#[derive(Debug, Default)]
pub enum SourceSearchState {
    ReceiveName { name: String },
    #[default]
    Done,
}

pub async fn receive_name_handler(name: String) -> ResponseResult<()> {
    unimplemented!()
}
