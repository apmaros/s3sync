use crate::model::photo::Photo;

pub struct ImageAlbum {
    pub(crate) name: String,
    pub(crate) photos: Vec<Photo>
}
