use crate::model::photo::Photo;

pub struct PhotoAlbum {
    pub(crate) name: String,
    pub(crate) photos: Vec<Photo>
}
